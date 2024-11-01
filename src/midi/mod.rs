//! Define midi devices and handle midi controls.

use anyhow::Result;
use device::{apc20::AkaiApc20, launch_control_xl::NovationLaunchControlXL};
use std::{cell::RefCell, fmt::Display, sync::mpsc::Sender};

use crate::channel::{ControlMessage as ChannelControlMessage, StateChange as ChannelStateChange};
use tunnels::{
    midi::{DeviceSpec, Event, Manager, Output},
    midi_controls::MidiDevice,
};

use crate::control::ControlMessage;

mod device;
mod mapping;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    Apc20(AkaiApc20),
    LaunchControlXL(NovationLaunchControlXL),
}

impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.device_name())
    }
}

impl MidiDevice for Device {
    fn device_name(&self) -> &str {
        match self {
            Self::Apc20(d) => d.device_name(),
            Self::LaunchControlXL(d) => d.device_name(),
        }
    }

    fn init_midi(&self, out: &mut tunnels::midi::Output<Self>) -> Result<()> {
        match self {
            Self::Apc20(d) => d.init_midi(out),
            Self::LaunchControlXL(d) => d.init_midi(out),
        }
    }
}

impl Device {
    /// Return all known midi device types.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Apc20(AkaiApc20 { channel_offset: 0 }),
            Self::LaunchControlXL(NovationLaunchControlXL { channel_offset: 0 }),
        ]
    }
}

impl MidiChannelController for Device {
    fn interpret(&self, event: &Event) -> Option<ChannelControlMessage> {
        match self {
            Self::Apc20(d) => d.interpret(event),
            Self::LaunchControlXL(d) => d.interpret(event),
        }
    }

    fn emit_channel_control(&self, msg: &ChannelStateChange, output: &mut Output<Device>) {
        match self {
            Self::Apc20(d) => d.emit_channel_control(msg, output),
            Self::LaunchControlXL(d) => d.emit_channel_control(msg, output),
        }
    }
}

/// MIDI handling, interpreting a MIDI event as a channel control message.
pub trait MidiChannelController {
    /// Interpet an incoming MIDI event as a channel control message.
    fn interpret(&self, event: &Event) -> Option<ChannelControlMessage>;

    /// Send MIDI state to handle the provided ChannelControlMessage.
    fn emit_channel_control(&self, msg: &ChannelStateChange, output: &mut Output<Device>);
}

pub struct MidiControlMessage {
    pub device: Device,
    pub event: Event,
}

/// Immutable-compatible wrapper around the midi manager.
/// Writing to a midi ouput requires a unique reference; we can safely wrap
/// this using RefCell since we only need a reference to the outputs to write,
/// and we can only be making one write call at a time.
pub struct MidiController(RefCell<Manager<Device>>);

impl MidiController {
    pub fn new(devices: Vec<DeviceSpec<Device>>, send: Sender<ControlMessage>) -> Result<Self> {
        let mut controller = Manager::default();
        for d in devices {
            controller.add_device(d, send.clone())?;
        }
        Ok(Self(RefCell::new(controller)))
    }

    /// Handle a channel state change message.
    pub fn update(&self, msg: &ChannelStateChange) {
        for output in self.0.borrow_mut().outputs() {
            // FIXME: tunnels devices are inside-out/stateless
            let device = *output.device();
            device.emit_channel_control(msg, output);
        }
    }
}

impl EmitMidiChannelMessage for MidiController {
    fn emit_midi_channel_message(&self, msg: &ChannelStateChange) {
        self.update(msg);
    }
}

pub trait EmitMidiChannelMessage {
    fn emit_midi_channel_message(&self, msg: &ChannelStateChange);
}
