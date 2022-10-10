use crossbeam_channel::{Receiver, Sender};
use log::{debug, error, warn};
use rosc::{OscMessage, OscPacket, OscType};
use std::collections::HashMap;
use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::thread;

struct ControlMessage;

type ControlMessageCreator = Box<dyn Fn(OscMessage) -> ControlMessage>;

pub struct ControlMap(HashMap<(String, String), ControlMessageCreator>);

impl ControlMap {
    fn new() -> Self {
        Self(HashMap::new())
    }
}

type ControlEvent = OscMessage;

/// Forward OSC messages to the provided sender.
/// Spawns a new thread to handle listening for messages.
fn start_listener<A: ToSocketAddrs>(
    addr: A,
    send: Sender<ControlEvent>,
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
fn forward_packet(packet: OscPacket, send: &Sender<ControlEvent>) {
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
