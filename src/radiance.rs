//! Control profile for a Radiance hazer.
//! Probably fine for any generic 2-channel hazer.

use log::debug;
use number::UnipolarFloat;

use crate::dmx::DmxAddr;
use crate::fixture::{ControlMessage as ShowControlMessage, EmitStateChange, Fixture};
use crate::util::unipolar_to_range;

pub struct Radiance {
    dmx_index: usize,
    haze: UnipolarFloat,
    fan: UnipolarFloat,
}

impl Radiance {
    pub fn new(dmx_addr: DmxAddr) -> Self {
        Self {
            dmx_index: dmx_addr - 1,
            haze: UnipolarFloat::ZERO,
            fan: UnipolarFloat::ZERO,
        }
    }

    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitStateChange) {
        use StateChange::*;
        match sc {
            Haze(v) => self.haze = v,
            Fan(v) => self.fan = v,
        };
        emitter.emit_radiance(sc);
    }
}

impl Fixture for Radiance {
    fn render(&self, dmx_univ: &mut [u8]) {
        dmx_univ[self.dmx_index] = unipolar_to_range(0, 255, self.haze);
        dmx_univ[self.dmx_index + 1] = unipolar_to_range(0, 255, self.fan);
        debug!("{:?}", &dmx_univ[self.dmx_index..self.dmx_index + 2]);
    }

    fn emit_state(&self, emitter: &mut dyn EmitStateChange) {
        use StateChange::*;
        emitter.emit_radiance(Haze(self.haze));
        emitter.emit_radiance(Fan(self.fan));
    }

    fn control(
        &mut self,
        msg: ShowControlMessage,
        emitter: &mut dyn EmitStateChange,
    ) -> Option<ShowControlMessage> {
        match msg {
            ShowControlMessage::Radiance(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    Haze(UnipolarFloat),
    Fan(UnipolarFloat),
}

// Venus has no controls that are not represented as state changes.
pub type ControlMessage = StateChange;
