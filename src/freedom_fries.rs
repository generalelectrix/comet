//! Basic control profile for 8-channel auto program control of the Chauvet
//! Freedom Stick.

//! Control profle for the Chauvet Rotosphere Q3, aka Son Of Spherion.

use log::{debug, error};
use number::UnipolarFloat;

use crate::dmx::DmxAddr;
use crate::fixture::{ControlMessage as ShowControlMessage, EmitStateChange, Fixture};
use crate::util::unipolar_to_range;

pub struct FreedomFries {
    dmx_index: usize,
    dimmer: UnipolarFloat,
    speed: UnipolarFloat,
    program: usize,
    program_cycle_all: bool,
}

impl FreedomFries {
    pub const CHANNEL_COUNT: usize = 8;
    pub const PROGRAM_COUNT: usize = 27;
    pub fn new(dmx_addr: DmxAddr) -> Self {
        Self {
            dmx_index: dmx_addr - 1,
            dimmer: UnipolarFloat::ZERO,
            speed: UnipolarFloat::ZERO,
            program: 0,
            program_cycle_all: false,
        }
    }

    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitStateChange) {
        use StateChange::*;
        match sc {
            Dimmer(v) => self.dimmer = v,
            Speed(v) => self.speed = v,
            Program(v) => {
                if v >= Self::PROGRAM_COUNT {
                    error!("Program select index {} out of range.", v);
                    return;
                }
                self.program = v;
            }
            ProgramCycleAll(v) => self.program_cycle_all = v,
        };
        emitter.emit_freedom_fries(sc);
    }
}

impl Fixture for FreedomFries {
    fn render(&self, dmx_univ: &mut [u8]) {
        let dmx_slice = &mut dmx_univ[self.dmx_index..self.dmx_index + Self::CHANNEL_COUNT];
        dmx_slice[0] = unipolar_to_range(0, 255, self.dimmer);
        dmx_slice[1] = 0;
        dmx_slice[2] = 0;
        dmx_slice[3] = 0;
        dmx_slice[4] = 0;
        dmx_slice[5] = 0; // TODO strobing
        dmx_slice[6] = if self.program_cycle_all {
            227
        } else {
            ((self.program * 8) + 11) as u8
        };
        dmx_slice[7] = unipolar_to_range(0, 255, self.speed);
        debug!("{:?}", dmx_slice);
    }

    fn emit_state(&self, emitter: &mut dyn EmitStateChange) {
        use StateChange::*;
        emitter.emit_freedom_fries(Dimmer(self.dimmer));
        emitter.emit_freedom_fries(Speed(self.speed));
        emitter.emit_freedom_fries(Program(self.program));
        emitter.emit_freedom_fries(ProgramCycleAll(self.program_cycle_all));
    }

    fn control(
        &mut self,
        msg: ShowControlMessage,
        emitter: &mut dyn EmitStateChange,
    ) -> Option<ShowControlMessage> {
        match msg {
            ShowControlMessage::FreedomFries(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    Dimmer(UnipolarFloat),
    Speed(UnipolarFloat),
    Program(usize),
    ProgramCycleAll(bool),
}

pub type ControlMessage = StateChange;
