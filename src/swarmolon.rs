//! Control profle for the Chauvet Swarm 5 FX, aka the Swarmolon.

use log::debug;
use number::UnipolarFloat;

use crate::dmx::DmxAddr;
use crate::fixture::{ControlMessage as ShowControlMessage, EmitStateChange, Fixture};
use crate::util::unipolar_to_range;

pub struct Swarmolon {
    dmx_indices: Vec<usize>,
}

impl Swarmolon {
    pub fn new(dmx_addrs: Vec<DmxAddr>) -> Self {
        Self {
            dmx_indices: dmx_addrs.iter().map(|a| a - 1).collect(),
        }
    }

    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitStateChange) {
        use StateChange::*;
        match sc {};
        emitter.emit_swarmolon(sc);
    }
}

impl Fixture for Swarmolon {
    fn render(&self, dmx_univ: &mut [u8]) {
        debug!("{:?}", &dmx_univ[self.dmx_index..self.dmx_index + 2]);
    }

    fn emit_state(&self, emitter: &mut dyn EmitStateChange) {
        use StateChange::*;
    }

    fn control(
        &mut self,
        msg: ShowControlMessage,
        emitter: &mut dyn EmitStateChange,
    ) -> Option<ShowControlMessage> {
        match msg {
            ShowControlMessage::Swarmolon(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {}

// No controls that are not represented as state changes.
pub type ControlMessage = StateChange;

pub enum DerbyColor {
    Red,
    Green,
    Blue,
    Amber,
    White,
}
