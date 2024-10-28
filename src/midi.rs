//! Define midi devices and handle midi controls.

use anyhow::Result;
use std::{fmt::Display, sync::mpsc::Sender};

use crate::channel::{
    ChannelControlMessage as ScopedChannelControlMessage, ControlMessage as ChannelControlMessage,
};
use tunnels::{
    midi::{DeviceSpec, Event, EventType, Manager},
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

    pub fn interpret(&self, event: &Event) -> Option<ChannelControlMessage> {
        match self {
            Self::AkaiApc20 => handle_apc20(event),
        }
    }
}

fn handle_apc20(event: &Event) -> Option<ChannelControlMessage> {
    // So far we only use upfaders and track select.
    // Should refactor this in the future to be more general.
    match event.mapping.event_type {
        EventType::ControlChange => {
            match event.mapping.control {
                0x7 => {
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
                0x33 => {
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
