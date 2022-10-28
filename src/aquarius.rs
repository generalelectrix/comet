//! Intuitive control profile for the American DJ Aquarius 250.

use std::time::Duration;

use log::debug;
use number::BipolarFloat;

use crate::dmx::DmxAddr;
use crate::fixture::{EmitStateChange as EmitShowStateChange, StateChange as ShowStateChange};
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

    pub fn update(&mut self, _: Duration) {}

    /// Render into the provided DMX universe.
    pub fn render(&self, dmx_univ: &mut [u8]) {
        for dmx_index in self.dmx_indices.iter() {
            dmx_univ[*dmx_index] = bipolar_to_split_range(self.rotation, 130, 8, 132, 255, 0);
            dmx_univ[*dmx_index + 1] = if self.lamp_on { 255 } else { 0 };
            debug!("{:?}", &dmx_univ[*dmx_index..*dmx_index + 2]);
        }
    }

    /// Emit the current value of all controllable state.
    pub fn emit_state<E: EmitStateChange>(&self, emitter: &mut E) {
        use StateChange::*;
        emitter.emit(LampOn(self.lamp_on));
        emitter.emit(Rotation(self.rotation));
    }

    pub fn control<E: EmitStateChange>(&mut self, msg: ControlMessage, emitter: &mut E) {
        self.handle_state_change(msg, emitter);
    }

    fn handle_state_change<E: EmitStateChange>(&mut self, sc: StateChange, emitter: &mut E) {
        use StateChange::*;
        match sc {
            LampOn(v) => self.lamp_on = v,
            Rotation(v) => self.rotation = v,
        };
        emitter.emit(sc);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    LampOn(bool),
    Rotation(BipolarFloat),
}

// Aquarius has no controls that are not represented as state changes.
pub type ControlMessage = StateChange;

pub trait EmitStateChange {
    fn emit(&mut self, sc: StateChange);
}

impl<T: EmitShowStateChange> EmitStateChange for T {
    fn emit(&mut self, sc: StateChange) {
        self.emit(ShowStateChange::Aquarius(sc));
    }
}
