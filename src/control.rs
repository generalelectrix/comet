//! Top-level traits and types for control events.

use std::{
    sync::mpsc::{channel, Receiver, RecvTimeoutError},
    time::Duration,
};

use anyhow::{bail, Result};
use prelude::OscControlMessage;
use tunnels::midi::CreateControlEvent;

use crate::{
    config::Config,
    midi::{Device, MidiControlMessage, MidiController},
    osc::{
        EmitOscMessage, EmitScopedOscMessage, OscClientId, OscController,
        OscMessageWithMetadataSender,
    },
};

/// Emit scoped control messages.
/// Will be extended in the future to potentially cover more cases.
pub trait EmitScopedControlMessage: EmitScopedOscMessage {}

impl<T> EmitScopedControlMessage for T where T: EmitScopedOscMessage {}

/// Emit control messages.
/// Will be extended in the future to potentially cover more cases.
pub trait EmitControlMessage: EmitOscMessage {}

impl<T> EmitControlMessage for T where T: EmitOscMessage {}

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
        &'a self,
        sender_id: Option<&'a OscClientId>,
    ) -> OscMessageWithMetadataSender<'_> {
        self.osc.sender_with_metadata(sender_id)
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

pub mod prelude {
    pub use super::*;
    pub use crate::osc::prelude::*;
}
