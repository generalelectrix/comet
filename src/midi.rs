//! Define midi devices and handle midi controls.

use anyhow::Result;
use std::{fmt::Display, sync::mpsc::Sender};

use crate::channel::{
    ChannelControlMessage as ScopedChannelControlMessage, ControlMessage as ChannelControlMessage,
    StateChange as ChannelStateChange,
};
use tunnels::{
    midi::{DeviceSpec, Event, EventType, Manager, Mapping, Output},
    midi_controls::{init_apc_20, unipolar_from_midi, MidiDevice},
};

use crate::show::ControlMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Device {
    AkaiApc20,
}

impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.device_name())
    }
}

impl MidiDevice for Device {
    fn device_name(&self) -> &str {
        match self {
            Self::AkaiApc20 => "Akai APC20",
        }
    }

    fn init_midi(&self, out: &mut tunnels::midi::Output<Self>) -> Result<()> {
        match self {
            Self::AkaiApc20 => init_apc_20(out),
        }
    }
}

impl Device {
    /// Return all known midi device types.
    pub fn all() -> Vec<Self> {
        vec![Self::AkaiApc20]
    }

    /// Interpet an incoming MIDI event as a channel control message.
    pub fn interpret(&self, event: &Event) -> Option<ChannelControlMessage> {
        match self {
            Self::AkaiApc20 => handle_apc20(event),
        }
    }

    /// Send MIDI state to handle the provided ChannelControlMessage.
    pub fn update(&self, msg: &ChannelStateChange, output: &mut Output<Self>) {
        match self {
            Self::AkaiApc20 => update_apc20(msg, output),
        }
    }
}

const APC20_FADER: u8 = 0x7;
const APC20_CHAN_SELECT: u8 = 0x33;

fn handle_apc20(event: &Event) -> Option<ChannelControlMessage> {
    // So far we only use upfaders and track select.
    // Should refactor this in the future to be more general.
    match event.mapping.event_type {
        EventType::ControlChange => {
            match event.mapping.control {
                APC20_FADER => {
                    // Upfader.
                    Some(ChannelControlMessage::Control {
                        channel_id: Some(event.mapping.channel as usize),
                        msg: ScopedChannelControlMessage::Level(unipolar_from_midi(event.value)),
                    })
                }
                _ => None,
            }
        }
        EventType::NoteOn => {
            match event.mapping.control {
                APC20_CHAN_SELECT => {
                    // Channel select button.
                    Some(ChannelControlMessage::SelectChannel(
                        event.mapping.channel as usize,
                    ))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn update_apc20(msg: &ChannelStateChange, output: &mut Output<Device>) {
    match msg {
        ChannelStateChange::SelectChannel(channel) => {
            let channel = channel.inner();
            if channel >= 8 {
                return;
            }
            for c in 0..8 {
                output.send(Event {
                    mapping: Mapping {
                        event_type: EventType::NoteOn,
                        channel: channel as u8,
                        control: APC20_CHAN_SELECT,
                    },
                    value: if c == channel { 127 } else { 0 },
                });
            }
        }
        _ => (),
    }
}

pub struct MidiControlMessage {
    pub device: Device,
    pub event: Event,
}

pub type MidiController = Manager<Device>;

pub fn init_midi_controller(
    devices: &[DeviceSpec<Device>],
    send: Sender<ControlMessage>,
) -> Result<MidiController> {
    let mut controller = MidiController::default();
    for d in devices {
        controller.add_device(d.clone(), send.clone())?;
    }
    Ok(controller)
}
