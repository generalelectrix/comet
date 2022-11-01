//! Intuitive control profile for the American DJ Aquarius 250.

use log::debug;
use number::BipolarFloat;

use crate::dmx::DmxAddr;
use crate::fixture::{ControlMessage as ShowControlMessage, EmitStateChange, Fixture};
use crate::util::bipolar_to_split_range;

/// Aggregate and control one or more Aquariuses.
pub struct Aquarius {
    /// Addresses of the H2Os under control.
    dmx_indices: Vec<usize>,
    lamp_on: bool,
    rotation: BipolarFloat,
}

impl Aquarius {
    pub fn new(dmx_addrs: Vec<DmxAddr>) -> Self {
        Self {
            dmx_indices: dmx_addrs.iter().map(|a| a - 1).collect(),
            lamp_on: false,
            rotation: BipolarFloat::ZERO,
        }
    }

    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitStateChange) {
        use StateChange::*;
        match sc {
            LampOn(v) => self.lamp_on = v,
            Rotation(v) => self.rotation = v,
        };
        emitter.emit_aquarius(sc);
    }
}

impl Fixture for Aquarius {
    fn render(&self, dmx_univ: &mut [u8]) {
        for dmx_index in self.dmx_indices.iter() {
            dmx_univ[*dmx_index] = bipolar_to_split_range(self.rotation, 130, 8, 132, 255, 0);
            dmx_univ[*dmx_index + 1] = if self.lamp_on { 255 } else { 0 };
            debug!("{:?}", &dmx_univ[*dmx_index..*dmx_index + 2]);
        }
    }

    fn emit_state(&self, emitter: &mut dyn EmitStateChange) {
        use StateChange::*;
        emitter.emit_aquarius(LampOn(self.lamp_on));
        emitter.emit_aquarius(Rotation(self.rotation));
    }

    fn control(
        &mut self,
        msg: ShowControlMessage,
        emitter: &mut dyn EmitStateChange,
    ) -> Option<ShowControlMessage> {
        match msg {
            ShowControlMessage::Aquarius(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    LampOn(bool),
    Rotation(BipolarFloat),
}

// Aquarius has no controls that are not represented as state changes.
pub type ControlMessage = StateChange;
