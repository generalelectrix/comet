//! Define midi devices and handle midi controls.

use anyhow::Result;
use std::sync::mpsc::Sender;

use tunnels::{
    midi::{DeviceSpec, Event, Manager},
    midi_controls::MidiDevice,
};

use crate::show::ControlMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Device {}

impl MidiDevice for Device {
    fn device_name(&self) -> &str {
        todo!()
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
        controller.add_device(d.clone(), send)?;
    }
    Ok(controller)
}
