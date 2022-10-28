use crossbeam_channel::{unbounded, Receiver, RecvTimeoutError, Sender};
use log::{error, info, warn};
use number::{BipolarFloat, UnipolarFloat};
use rosc::{encoder, OscMessage, OscPacket, OscType};
use simple_error::bail;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use crate::fixture::{ControlMessage, EmitStateChange, StateChange};

mod comet;
mod lumasphere;

pub struct OscController {
    control_map: ControlMap<ControlMessage>,
    recv: Receiver<OscMessage>,
    send: Sender<OscMessage>,
}

impl OscController {
    pub fn new(receive_port: u16, send_host: &str, send_port: u16) -> Result<Self, Box<dyn Error>> {
        let recv_addr = SocketAddr::from_str(&format!("0.0.0.0:{}", receive_port))?;
        let send_adr = SocketAddr::from_str(&format!("{}:{}", send_host, send_port))?;
        let control_recv = start_listener(recv_addr)?;
        let response_send = start_sender(send_adr)?;
        Ok(Self {
            control_map: ControlMap::new(),
            recv: control_recv,
            send: response_send,
        })
    }

    pub fn map_comet_controls(&mut self) {
        comet::map_controls(&mut self.control_map);
    }

    pub fn map_lumasphere_controls(&mut self) {
        lumasphere::map_controls(&mut self.control_map);
    }

    pub fn recv(&self, timeout: Duration) -> Result<Option<ControlMessage>, Box<dyn Error>> {
        let msg = match self.recv.recv_timeout(timeout) {
            Ok(msg) => msg,
            Err(RecvTimeoutError::Timeout) => {
                return Ok(None);
            }
            Err(RecvTimeoutError::Disconnected) => {
                bail!("OSC receiver disconnected");
            }
        };
        self.control_map.handle(msg)
    }
}

impl EmitStateChange for OscController {
    fn emit(&mut self, sc: StateChange) {
        match sc {
            StateChange::Comet(sc) => comet::handle_state_change(sc, &mut |msg| {
                let _ = self.send.send(msg);
            }),
            StateChange::Lumasphere(sc) => lumasphere::handle_state_change(sc, &mut |msg| {
                let _ = self.send.send(msg);
            }),
        }
    }
}

/// Unpack the group and control from the provided address.
/// FIXME: refactor how we key the control map to avoid the need to key with
/// owned strings.
fn get_group_control(addr: &str) -> Result<(String, String), OscError> {
    let mut pieces_iter = addr.split("/").skip(1);
    let group = pieces_iter.next().ok_or_else(|| OscError {
        addr: addr.to_string(),
        msg: "group is missing".to_string(),
    })?;
    let control = pieces_iter.next().ok_or_else(|| OscError {
        addr: addr.to_string(),
        msg: "control is missing".to_string(),
    })?;
    Ok((group.to_string(), control.to_string()))
}

type ControlMessageCreator<C> = Box<dyn Fn(OscMessage) -> Result<Option<C>, Box<dyn Error>>>;

pub struct ControlMap<C>(HashMap<(String, String), ControlMessageCreator<C>>);

impl<C> ControlMap<C> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn handle(&self, msg: OscMessage) -> Result<Option<C>, Box<dyn Error>> {
        let key = get_group_control(&msg.addr)?;
        match self.0.get(&key) {
            None => {
                warn!("No control handler matched address \"{}\".", msg.addr);
                Ok(None)
            }
            Some(handler) => handler(msg),
        }
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

    pub fn add_radio_button_array<F>(&mut self, rb: RadioButton, process: F)
    where
        F: Fn(usize) -> C + 'static,
    {
        self.add_fetch_process(
            rb.group,
            rb.control,
            move |m| rb.parse(m),
            move |x| Some(process(x)),
        )
    }
}

/// Forward OSC messages to the provided sender.
/// Spawns a new thread to handle listening for messages.
fn start_listener(addr: SocketAddr) -> Result<Receiver<OscMessage>, Box<dyn Error>> {
    let (send, recv) = unbounded();
    let socket = UdpSocket::bind(addr)?;

    let mut buf = [0u8; rosc::decoder::MTU];

    let mut recv_packet = move || -> Result<OscPacket, Box<dyn Error>> {
        let size = socket.recv(&mut buf)?;
        let (_, packet) = rosc::decoder::decode_udp(&buf[..size])?;
        Ok(packet)
    };

    thread::spawn(move || loop {
        match recv_packet() {
            Ok(packet) => {
                forward_packet(packet, &send);
            }
            Err(e) => {
                error!("Error receiving from OSC input: {}", e);
            }
        }
    });
    Ok(recv)
}

/// Drain a control channel of OSC messages and send them.
/// Assumes we only have one controller.
fn start_sender(addr: SocketAddr) -> Result<Sender<OscMessage>, Box<dyn Error>> {
    let (send, recv) = unbounded();
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    let send_packet = move |msg| -> Result<(), Box<dyn Error>> {
        let msg_buf = encoder::encode(&OscPacket::Message(msg))?;
        socket.send_to(&msg_buf, addr)?;
        Ok(())
    };

    thread::spawn(move || loop {
        let msg = match recv.recv() {
            Err(_) => {
                info!("OSC sender channel hung up, terminating sender thread.");
                return;
            }
            Ok(m) => m,
        };
        if let Err(e) = send_packet(msg) {
            error!("OSC send error: {}.", e);
        }
    });
    Ok(send)
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
/// Model a 1D button grid with radio-select behavior.
/// This implements the TouchOSC model for a button grid.
/// Special-cased to handle only 1D grids.
#[derive(Clone)]
pub struct RadioButton {
    group: &'static str,
    control: &'static str,
    n: usize,
}

impl RadioButton {
    /// Get a index from a collection of radio buttons, mapped to numeric addresses.
    pub fn parse(&self, v: OscMessage) -> Result<usize, OscError> {
        let (x, y) = match parse_radio_button_indices(&v.addr) {
            Ok(indices) => indices,
            Err(err) => {
                return Err(OscError {
                    addr: v.addr,
                    msg: err,
                });
            }
        };
        if x >= self.n {
            return Err(OscError {
                addr: v.addr,
                msg: format!("radio button x index out of range: {}", x),
            });
        }
        if y > 0 {
            return Err(OscError {
                addr: v.addr,
                msg: format!("radio button y index out of range: {}", y),
            });
        }
        Ok(x)
    }

    pub fn set<S>(&self, n: usize, send: &mut S) -> Result<(), Box<dyn Error>>
    where
        S: FnMut(OscMessage),
    {
        if n >= self.n {
            bail!(
                "radio button index {} out of range for {}/{}",
                n,
                self.group,
                self.control
            );
        }
        for i in 0..self.n {
            let val = if i == n { 1.0 } else { 0.0 };
            send(OscMessage {
                addr: format!("/{}/{}/{}/1", self.group, self.control, i + 1),
                args: vec![OscType::Float(val)],
            })
        }
        Ok(())
    }
}

/// Parse radio button indices from a TouchOSC button grid.
fn parse_radio_button_indices(addr: &str) -> Result<(usize, usize), String> {
    let mut pieces_iter = addr.split("/").skip(3).take(2).map(str::parse::<usize>);
    let x = pieces_iter
        .next()
        .ok_or_else(|| "x radio button index missing".to_string())?
        .map_err(|err| format!("failed to parse radio button x index: {}", err))?;
    let y = pieces_iter
        .next()
        .ok_or_else(|| "y radio button index missing".to_string())?
        .map_err(|err| format!("failed to parse radio button y index: {}", err))?;
    if x == 0 {
        return Err(format!("x index is unexpectedly 0"));
    }
    if y == 0 {
        return Err(format!("y index is unexpectedly 0"));
    }
    Ok((x - 1, y - 1))
}
