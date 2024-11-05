//! Top-level traits and types for control events.

use std::{
    sync::mpsc::{channel, Receiver, RecvTimeoutError},
    time::Duration,
};

use anyhow::{bail, Result};
use rosc::OscMessage;
use tunnels::midi::CreateControlEvent;

use crate::{
    config::Config,
    midi::{
        Device, EmitMidiChannelMessage, EmitMidiMasterMessage, MidiControlMessage, MidiController,
    },
    osc::{
        EmitOscMessage, EmitScopedOscMessage, OscClientId, OscControlMessage, OscControlResponse,
        OscController, ScopedControlEmitter, TalkbackMode,
    },
};

/// Emit scoped control messages.
/// Will be extended in the future to potentially cover more cases.
pub trait EmitScopedControlMessage: EmitScopedOscMessage {}

impl<T> EmitScopedControlMessage for T where T: EmitScopedOscMessage {}

/// Emit control messages.
/// Will be extended in the future to potentially cover more cases.
pub trait EmitControlMessage:
    EmitOscMessage + EmitMidiChannelMessage + EmitMidiMasterMessage
{
}

impl<T> EmitControlMessage for T where
    T: EmitOscMessage + EmitMidiChannelMessage + EmitMidiMasterMessage
{
}

/// Handle receiving and responding to show control messages.
pub struct Controller {
    osc: OscController,
    midi: MidiController,
    recv: Receiver<ControlMessage>,
}

impl Controller {
    pub fn from_config(cfg: &Config) -> Result<Self> {
        let (send, recv) = channel();
        Ok(Self {
            osc: OscController::new(cfg.receive_port, cfg.controllers.clone(), send.clone())?,
            midi: MidiController::new(cfg.midi_devices.clone(), send)?,
            recv,
        })
    }

    pub fn recv(&self, timeout: Duration) -> Result<Option<ControlMessage>> {
        match self.recv.recv_timeout(timeout) {
            Ok(msg) => Ok(Some(msg)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => {
                bail!("control receiver disconnected");
            }
        }
    }

    /// Return a decorated version of self that will include the provided
    /// metadata when sending OSC response messages.
    pub fn sender_with_metadata<'a>(
        &'a mut self,
        sender_id: Option<&'a OscClientId>,
    ) -> ControlMessageWithMetadataSender<'_> {
        ControlMessageWithMetadataSender {
            sender_id,
            controller: self,
        }
    }
}

/// Decorate the Controller to add message metedata to control responses.
pub struct ControlMessageWithMetadataSender<'a> {
    pub sender_id: Option<&'a OscClientId>,
    pub controller: &'a mut Controller,
}

impl<'a> EmitOscMessage for ControlMessageWithMetadataSender<'a> {
    fn emit_osc(&self, msg: OscMessage) {
        self.controller.osc.send(OscControlResponse {
            sender_id: self.sender_id.cloned(),
            talkback: TalkbackMode::All, // FIXME: hardcoded talkback
            msg,
        });
    }
}

impl<'a> EmitMidiChannelMessage for ControlMessageWithMetadataSender<'a> {
    fn emit_midi_channel_message(&self, msg: &crate::channel::StateChange) {
        self.controller.midi.emit_channel_control(msg);
    }
}

impl<'a> EmitMidiMasterMessage for ControlMessageWithMetadataSender<'a> {
    fn emit_midi_master_message(&self, msg: &crate::master::StateChange) {
        self.controller.midi.emit_master_control(msg);
    }
}

pub enum ControlMessage {
    Osc(OscControlMessage),
    Midi(MidiControlMessage),
}

impl CreateControlEvent<Device> for ControlMessage {
    fn from_event(event: tunnels::midi::Event, device: Device) -> Self {
        Self::Midi(MidiControlMessage { device, event })
    }
}
