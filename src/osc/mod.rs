use crate::fixture::FixtureGroupKey;
use crate::fixture::{ControlMessage, ControlMessagePayload, FixtureType, GroupName};
use anyhow::bail;
use anyhow::Result;
use control_message::OscControlMessage;
use log::{error, info};
use number::{BipolarFloat, Phase, UnipolarFloat};
use rosc::{encoder, OscMessage, OscPacket, OscType};
use serde::Deserialize;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Display;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;
use thiserror::Error;

use self::radio_button::{EnumRadioButton, RadioButton};

mod animation;
mod basic_controls;
mod channels;
mod control_message;
mod fader_array;
mod label_array;
mod master;
mod profile;
mod radio_button;
mod register;

pub use register::prompt_osc_config;

/// Map OSC control inputs for a fixture type.
pub trait MapControls {
    /// Add OSC control mappings to the provided control map.
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>);

    /// Return aliases for fixture type, if this is a fixture.
    /// Return None if we're not mapping fixture controls.
    fn fixture_type_aliases(&self) -> Vec<(String, FixtureType)>;
}

/// Emit control messages.
/// Will be extended in the future to potentially cover more cases.
pub trait EmitControlMessage: EmitOscMessage {}

pub trait EmitOscMessage {
    fn emit_osc(&self, msg: OscMessage);
}

impl<T> EmitControlMessage for T where T: EmitOscMessage {}

/// Process a state change message into OSC messages.
pub trait HandleOscStateChange<SC> {
    /// Convert the provided state change into OSC messages and send them.
    fn emit_osc_state_change<S>(_sc: SC, _send: &S)
    where
        S: EmitOscMessage + ?Sized,
    {
    }
}

/// Process a state change message into control state changes.
pub trait HandleStateChange<SC>: HandleOscStateChange<SC> {
    fn emit<S>(sc: SC, send: &S)
    where
        S: EmitControlMessage + ?Sized,
    {
        Self::emit_osc_state_change(sc, send);
    }
}

impl<T, SC> HandleStateChange<SC> for T where T: HandleOscStateChange<SC> {}

pub struct OscController {
    control_map: ControlMap<ControlMessagePayload>,
    key_map: HashMap<String, FixtureType>,
    recv: Receiver<OscControlMessage>,
    send: Sender<OscControlResponse>,
}

impl OscController {
    pub fn new(receive_port: u16, send_addrs: Vec<OscClientId>) -> Result<Self> {
        let recv_addr = SocketAddr::from_str(&format!("0.0.0.0:{}", receive_port))?;
        let control_recv = start_listener(recv_addr)?;
        let response_send = start_sender(send_addrs)?;
        Ok(Self {
            control_map: ControlMap::new(),
            key_map: HashMap::new(),
            recv: control_recv,
            send: response_send,
        })
    }

    pub fn recv(&self, timeout: Duration) -> Result<Option<ControlMessage>> {
        let msg = match self.recv.recv_timeout(timeout) {
            Ok(msg) => msg,
            Err(RecvTimeoutError::Timeout) => {
                return Ok(None);
            }
            Err(RecvTimeoutError::Disconnected) => {
                bail!("OSC receiver disconnected");
            }
        };
        Ok(self
            .control_map
            .handle(&msg)?
            .map(|(m, talkback)| ControlMessage {
                sender_id: msg.client_id,
                talkback,
                msg: m,
                key: self
                    .key_map
                    .get(msg.entity_type())
                    .map(|fixture| FixtureGroupKey {
                        fixture: *fixture,
                        group: msg.group,
                    }),
            }))
    }

    pub fn map_controls<M: MapControls>(&mut self, fixture: &M) {
        fixture.map_controls(&mut self.control_map);
        for (control_key, fixture_type) in fixture.fixture_type_aliases() {
            match self.key_map.entry(control_key) {
                Entry::Vacant(e) => {
                    e.insert(fixture_type);
                }
                Entry::Occupied(existing) => {
                    assert!(
                        existing.get() == &fixture_type,
                        "fixture type alias conflict for {}: {}, {}",
                        existing.key(),
                        existing.get(),
                        fixture_type
                    );
                }
            }
        }
    }

    /// Return a decorated version of self that will include the provided
    /// metadata when sending OSC response messages.
    pub fn sender_with_metadata<'a>(
        &'a self,
        sender_id: Option<&'a OscClientId>,
        talkback: TalkbackMode,
    ) -> OscMessageWithMetadataSender<'_> {
        OscMessageWithMetadataSender {
            sender_id,
            talkback,
            controller: self,
        }
    }
}

/// Decorate the OscController to add message metedata to control responses.
pub struct OscMessageWithMetadataSender<'a> {
    pub sender_id: Option<&'a OscClientId>,
    pub talkback: TalkbackMode,
    pub controller: &'a OscController,
}

impl<'a> EmitOscMessage for OscMessageWithMetadataSender<'a> {
    fn emit_osc(&self, msg: OscMessage) {
        if self
            .controller
            .send
            .send(OscControlResponse {
                sender_id: self.sender_id.cloned(),
                talkback: self.talkback,
                msg,
            })
            .is_err()
        {
            error!("OSC send channel is disconnected.");
        }
    }
}

/// Decorate a control message emitter to inject a group into the address.
pub struct OscMessageWithGroupSender<'a> {
    pub group: Option<&'a GroupName>,
    pub emitter: &'a dyn EmitControlMessage,
}

impl<'a> EmitOscMessage for OscMessageWithGroupSender<'a> {
    fn emit_osc(&self, mut msg: OscMessage) {
        if let Some(g) = &self.group {
            // If a group is set, prepend the ID to the address.
            // FIXME: would be nice to think through this a bit and see if
            // we can avoid this allocation by somehow transparently threading
            // the group into the send call via something like constructor
            // injection.
            msg.addr = format!("/:{}{}", g, msg.addr);
        }
        self.emitter.emit_osc(msg);
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Deserialize)]
pub struct OscClientId(SocketAddr);

impl OscClientId {
    pub fn addr(&self) -> &SocketAddr {
        &self.0
    }
}

type ControlMessageCreator<C> =
    Box<dyn Fn(&OscControlMessage) -> Result<Option<(C, TalkbackMode)>>>;

pub struct ControlMap<C>(HashMap<String, ControlMessageCreator<C>>);

pub type FixtureControlMap = ControlMap<ControlMessagePayload>;

impl Display for OscClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OSC client at {}", self.0)
    }
}

impl<C> ControlMap<C> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn handle(&self, msg: &OscControlMessage) -> Result<Option<(C, TalkbackMode)>> {
        let key = msg.control_key();
        match self.0.get(key) {
            None => {
                bail!("No control handler matched key \"{}\".", key);
            }
            Some(handler) => handler(msg),
        }
    }

    pub fn add<F>(&mut self, group: &str, control: &str, handler: F)
    where
        F: Fn(&OscControlMessage) -> Result<Option<C>> + 'static,
    {
        let key = format!("/{}/{}", group, control);
        match self.0.entry(key) {
            Entry::Occupied(e) => {
                let key = e.key();
                panic!("Duplicate control definition for {}", key,)
            }
            Entry::Vacant(v) => v.insert(Box::new(move |m| {
                Ok(handler(m)?.map(|msg| (msg, TalkbackMode::All)))
            })),
        };
    }

    pub fn add_fetch_process<F, T, P>(&mut self, group: &str, control: &str, fetch: F, process: P)
    where
        F: Fn(&OscControlMessage) -> Result<T, OscError> + 'static,
        P: Fn(T) -> Option<C> + 'static,
    {
        self.add(group, control, move |v| Ok(process(fetch(v)?)))
    }

    pub fn add_unipolar<F>(&mut self, group: &str, control: &str, process: F)
    where
        F: Fn(UnipolarFloat) -> C + 'static,
    {
        self.add_fetch_process(group, control, get_unipolar, move |v| Some(process(v)))
    }

    pub fn add_bipolar<F>(&mut self, group: &str, control: &str, process: F)
    where
        F: Fn(BipolarFloat) -> C + 'static,
    {
        self.add_fetch_process(group, control, get_bipolar, move |v| Some(process(v)))
    }

    pub fn add_phase<F>(&mut self, group: &str, control: &str, process: F)
    where
        F: Fn(Phase) -> C + 'static,
    {
        self.add_fetch_process(group, control, get_phase, move |v| Some(process(v)))
    }

    pub fn add_bool<F>(&mut self, group: &str, control: &str, process: F)
    where
        F: Fn(bool) -> C + 'static,
    {
        self.add_fetch_process(group, control, get_bool, move |v| Some(process(v)))
    }

    /// Add a collection of control actions for each variant of the specified enum type.
    pub fn add_enum_handler<EnumType, Parse, Process, ParseResult>(
        &mut self,
        group: &str,
        control: &str,
        parse: Parse,
        process: Process,
    ) where
        EnumType: EnumRadioButton,
        <EnumType as FromStr>::Err: std::fmt::Display,
        Parse: Fn(&OscControlMessage) -> Result<ParseResult, OscError> + 'static,
        Process: Fn(EnumType, ParseResult) -> C + 'static,
    {
        self.add(group, control, move |m| {
            let variant: EnumType = EnumType::parse(m)?;
            let val = parse(m)?;
            Ok(Some(process(variant, val)))
        })
    }
}

/// Forward OSC messages to the provided sender.
/// Spawns a new thread to handle listening for messages.
fn start_listener(addr: SocketAddr) -> Result<Receiver<OscControlMessage>> {
    let (send, recv) = channel();
    let socket = UdpSocket::bind(addr)?;

    let mut buf = [0u8; rosc::decoder::MTU];

    let mut recv_packet = move || -> Result<_> {
        let (size, sender_addr) = socket.recv_from(&mut buf)?;
        let (_, packet) = rosc::decoder::decode_udp(&buf[..size])?;
        Ok((packet, OscClientId(sender_addr)))
    };

    thread::spawn(move || loop {
        let (packet, client_id) = match recv_packet() {
            Ok(msg) => msg,
            Err(e) => {
                error!("Error receiving from OSC input: {}", e);
                continue;
            }
        };
        if let Err(e) = forward_packet(packet, client_id, &send) {
            error!("Error unpacking/forwarding OSC packet: {}", e);
        }
    });
    Ok(recv)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TalkbackMode {
    /// Control responses should be sent to all clients.
    All,
    /// Control responses should be sent to all clients except the sender.
    Off,
}

pub struct OscControlResponse {
    sender_id: Option<OscClientId>,
    talkback: TalkbackMode,
    msg: OscMessage,
}

/// Drain a control channel of OSC messages and send them.
/// Sends each message to every provided address, unless the talkback mode
/// says otherwise.
fn start_sender(clients: Vec<OscClientId>) -> Result<Sender<OscControlResponse>> {
    let (send, recv) = channel::<OscControlResponse>();
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    thread::spawn(move || loop {
        let Ok(resp) = recv.recv() else {
            info!("OSC sender channel hung up, terminating sender thread.");
            return;
        };
        // Encode the message.
        let packet = OscPacket::Message(resp.msg);
        let msg_buf = match encoder::encode(&packet) {
            Ok(buf) => buf,
            Err(err) => {
                error!("Error encoding OSC packet {packet:?}: {err}.");
                continue;
            }
        };
        // debug!("Sending OSC message: {:?}", msg);
        for client in &clients {
            if resp.talkback == TalkbackMode::Off && resp.sender_id == Some(*client) {
                continue;
            }
            if let Err(err) = socket.send_to(&msg_buf, client.addr()) {
                error!("OSC send error to {client}: {}.", err);
            }
        }
    });
    Ok(send)
}

/// Recursively unpack OSC packets and send all the inner messages as control events.
fn forward_packet(
    packet: OscPacket,
    client_id: OscClientId,
    send: &Sender<OscControlMessage>,
) -> Result<(), OscError> {
    match packet {
        OscPacket::Message(m) => {
            // info!("Received OSC message: {:?}", m);
            // Set TouchOSC pages to send this message, and ignore them all here.
            if m.addr == "/page" {
                return Ok(());
            }
            let cm = OscControlMessage::new(m, client_id)?;
            send.send(cm).unwrap();
        }
        OscPacket::Bundle(msgs) => {
            for subpacket in msgs.content {
                forward_packet(subpacket, client_id, send)?;
            }
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
#[error("{addr}: {msg}")]
pub struct OscError {
    pub addr: String,
    pub msg: String,
}

/// Get a single float argument from the provided OSC message.
fn get_float(v: &OscControlMessage) -> Result<f64, OscError> {
    match &v.arg {
        OscType::Float(v) => Ok(*v as f64),
        OscType::Double(v) => Ok(*v),
        other => Err(v.err(format!(
            "expected a single float argument but found {:?}",
            other
        ))),
    }
}

/// Get a single unipolar float argument from the provided OSC message.
fn get_unipolar(v: &OscControlMessage) -> Result<UnipolarFloat, OscError> {
    Ok(UnipolarFloat::new(get_float(v)?))
}

/// Get a single bipolar float argument from the provided OSC message.
fn get_bipolar(v: &OscControlMessage) -> Result<BipolarFloat, OscError> {
    Ok(BipolarFloat::new(get_float(v)?))
}

/// Get a single phase argument from the provided OSC message.
fn get_phase(v: &OscControlMessage) -> Result<Phase, OscError> {
    Ok(Phase::new(get_float(v)?))
}

fn quadratic(v: UnipolarFloat) -> UnipolarFloat {
    UnipolarFloat::new(v.val().powi(2))
}

/// Get a single boolean argument from the provided OSC message.
/// Coerce ints and floats to boolean values.
fn get_bool(v: &OscControlMessage) -> Result<bool, OscError> {
    let bval = match &v.arg {
        OscType::Bool(b) => *b,
        OscType::Int(i) => *i != 0,
        OscType::Float(v) => *v != 0.0,
        OscType::Double(v) => *v != 0.0,
        other => {
            return Err(v.err(format!(
                "expected a single bool argument but found {:?}",
                other
            )));
        }
    };
    Ok(bval)
}

/// A OSC message processor that ignores the message payload, returning unit.
fn ignore_payload(_: &OscControlMessage) -> Result<(), OscError> {
    Ok(())
}

/// Send an OSC message setting the state of a float control.
fn send_float<S, V: Into<f64>>(group: &str, control: &str, val: V, emitter: &S)
where
    S: crate::osc::EmitOscMessage + ?Sized,
{
    emitter.emit_osc(OscMessage {
        addr: format!("/{group}/{control}"),
        args: vec![OscType::Float(val.into() as f32)],
    });
}
