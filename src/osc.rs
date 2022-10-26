use crossbeam_channel::{Receiver, Sender};
use log::{debug, error, warn};
use number::{BipolarFloat, UnipolarFloat};
use rosc::{OscMessage, OscPacket, OscType};
use simple_error::{bail, SimpleError};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::thread;

use crate::comet::ControlMessage;

mod comet;
mod lumasphere;

type ControlMessageCreator<C> = Box<dyn Fn(OscMessage) -> Result<Option<C>, Box<dyn Error>>>;

pub struct ControlMap<C>(HashMap<(String, String), ControlMessageCreator<C>>);

impl<C> ControlMap<C> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add<F, Group, Control>(&mut self, group: Group, control: Control, handler: F)
    where
        F: Fn(OscMessage) -> Result<Option<C>, Box<dyn Error>> + 'static,
        Group: Into<String> + Display,
        Control: Into<String> + Display,
    {
        match self.0.entry((group.into(), control.into())) {
            Entry::Occupied(e) => {
                let key = e.key();
                panic!("Duplicate control definition for ({}, {}).", key.0, key.1)
            }
            Entry::Vacant(v) => v.insert(Box::new(handler)),
        };
    }

    pub fn add_fetch_process<F, T, P, Group, Control>(
        &mut self,
        group: Group,
        control: Control,
        fetch: F,
        process: P,
    ) where
        F: Fn(OscMessage) -> Result<T, OscError> + 'static,
        P: Fn(T) -> Option<C> + 'static,
        Group: Into<String> + Display,
        Control: Into<String> + Display,
    {
        self.add(group, control, move |v| Ok(process(fetch(v)?)))
    }

    pub fn add_unipolar<F, Group, Control>(&mut self, group: Group, control: Control, process: F)
    where
        F: Fn(UnipolarFloat) -> C + 'static,
        Group: Into<String> + Display,
        Control: Into<String> + Display,
    {
        self.add_fetch_process(group, control, get_unipolar, move |v| Some(process(v)))
    }

    pub fn add_bipolar<F, Group, Control>(&mut self, group: Group, control: Control, process: F)
    where
        F: Fn(BipolarFloat) -> C + 'static,
        Group: Into<String> + Display,
        Control: Into<String> + Display,
    {
        self.add_fetch_process(group, control, get_bipolar, move |v| Some(process(v)))
    }

    pub fn add_bool<F, Group, Control>(&mut self, group: Group, control: Control, process: F)
    where
        F: Fn(bool) -> C + 'static,
        Group: Into<String> + Display,
        Control: Into<String> + Display,
    {
        self.add_fetch_process(group, control, get_bool, move |v| Some(process(v)))
    }

    pub fn add_trigger<Group, Control>(&mut self, group: Group, control: Control, event: C)
    where
        C: Copy + 'static,
        Group: Into<String> + Display,
        Control: Into<String> + Display,
    {
        self.add_fetch_process(
            group,
            control,
            get_bool,
            move |v| {
                if v {
                    Some(event)
                } else {
                    None
                }
            },
        )
    }

    pub fn add_1d_radio_button_array<F, Group, Control>(
        &mut self,
        group: Group,
        control: Control,
        process: F,
    ) where
        F: Fn(usize) -> C + 'static,
        Group: Into<String> + Display,
        Control: Into<String> + Display,
    {
        self.add_fetch_process(group, control, radio_button, move |(x, _)| Some(process(x)))
    }
}

/// Forward OSC messages to the provided sender.
/// Spawns a new thread to handle listening for messages.
fn start_listener<A: ToSocketAddrs>(
    addr: A,
    send: Sender<OscMessage>,
) -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind(addr)?;

    let mut buf = [0u8; rosc::decoder::MTU];

    let mut recv = move || -> Result<OscPacket, Box<dyn Error>> {
        let size = socket.recv(&mut buf)?;
        let (_, packet) = rosc::decoder::decode_udp(&buf[..size])?;
        Ok(packet)
    };

    thread::spawn(move || loop {
        match recv() {
            Ok(packet) => {
                forward_packet(packet, &send);
            }
            Err(e) => {
                error!("Error receiving from OSC input: {}", e);
            }
        }
    });
    Ok(())
}

/// Recursively unpack OSC packets and send all the inner messages as control events.
fn forward_packet(packet: OscPacket, send: &Sender<OscMessage>) {
    match packet {
        OscPacket::Message(m) => {
            send.send(m).unwrap();
        }
        OscPacket::Bundle(msgs) => {
            for subpacket in msgs.content {
                forward_packet(subpacket, send);
            }
        }
    }
}

#[derive(Debug)]
pub struct OscError {
    pub addr: String,
    pub msg: String,
}

impl Display for OscError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.addr, self.msg)
    }
}

impl Error for OscError {}

/// Get a single OSC argument from the provided OSC message.
fn get_single_arg(mut v: OscMessage) -> Result<(String, OscType), OscError> {
    if v.args.len() != 1 {
        return Err(OscError {
            msg: format!("expected 1 argument but has {}: {:?}", v.args.len(), v.args),
            addr: v.addr,
        });
    }
    Ok((v.addr, v.args.pop().unwrap()))
}

/// Get a single float argument from the provided OSC message.
fn get_float(v: OscMessage) -> Result<f64, OscError> {
    let (addr, arg) = get_single_arg(v)?;
    match arg {
        OscType::Float(v) => Ok(v as f64),
        OscType::Double(v) => Ok(v),
        other => Err(OscError {
            addr,
            msg: format!("expected a single float argument but found {:?}", other),
        }),
    }
}

/// Get a single unipolar float argument from the provided OSC message.
fn get_unipolar(v: OscMessage) -> Result<UnipolarFloat, OscError> {
    Ok(UnipolarFloat::new(get_float(v)?))
}

/// Get a single bipolar float argument from the provided OSC message.
fn get_bipolar(v: OscMessage) -> Result<BipolarFloat, OscError> {
    Ok(BipolarFloat::new(get_float(v)?))
}

fn quadratic(v: UnipolarFloat) -> UnipolarFloat {
    UnipolarFloat::new(v.val().powi(2))
}

/// Get a single boolean argument from the provided OSC message.
/// Coerce ints and floats to boolean values.
fn get_bool(v: OscMessage) -> Result<bool, OscError> {
    let (addr, arg) = get_single_arg(v)?;
    let bval = match arg {
        OscType::Bool(b) => b,
        OscType::Int(i) => i != 0,
        OscType::Float(v) => v != 0.0,
        OscType::Double(v) => v != 0.0,
        other => {
            return Err(OscError {
                addr,
                msg: format!("expected a single bool argument but found {:?}", other),
            })
        }
    };
    Ok(bval)
}

/// Get a index from a collection of radio buttons, mapped to numeric addresses.
/// This implements the TouchOSC model for a button grid.
fn radio_button(v: OscMessage) -> Result<(usize, usize), OscError> {
    let parsed = v
        .addr
        .split("/")
        .skip(3)
        .map(str::parse::<usize>)
        .take(2)
        .collect::<Result<(Vec<_>), _>>();

    let parsed = match parsed {
        Err(e) => {
            return Err(OscError {
                addr: v.addr,
                msg: format!("failed to parse radio button index: {}", e),
            })
        }
        Ok(v) => v,
    };
    if parsed.len() != 2 {
        return Err(OscError {
            addr: v.addr,
            msg: format!("expected two radio button indexes, got {:?}", parsed),
        });
    }
    Ok((parsed[0], parsed[1]))
}
