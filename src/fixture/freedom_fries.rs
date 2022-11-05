//! Basic control profile for 8-channel auto program control of the Chauvet
//! Freedom Stick.

//! Control profle for the Chauvet Rotosphere Q3, aka Son Of Spherion.

use log::error;
use number::UnipolarFloat;

use super::{EmitFixtureStateChange, Fixture, FixtureControlMessage, PatchFixture};
use crate::{master::MasterControls, util::unipolar_to_range};

#[derive(Default, Debug)]
pub struct FreedomFries {
    dimmer: UnipolarFloat,
    speed: UnipolarFloat,
    program: usize,
    program_cycle_all: bool,
}

impl PatchFixture for FreedomFries {
    fn channel_count(&self) -> usize {
        8
    }
}

impl FreedomFries {
    pub const PROGRAM_COUNT: usize = 27;
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
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
    fn render(&self, _master_controls: &MasterControls, dmx_buf: &mut [u8]) {
        dmx_buf[0] = unipolar_to_range(0, 255, self.dimmer);
        dmx_buf[1] = 0;
        dmx_buf[2] = 0;
        dmx_buf[3] = 0;
        dmx_buf[4] = 0;
        dmx_buf[5] = 0; // TODO strobing
        dmx_buf[6] = if self.program_cycle_all {
            227
        } else {
            ((self.program * 8) + 11) as u8
        };
        dmx_buf[7] = unipolar_to_range(0, 255, self.speed);
    }

    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        emitter.emit_freedom_fries(Dimmer(self.dimmer));
        emitter.emit_freedom_fries(Speed(self.speed));
        emitter.emit_freedom_fries(Program(self.program));
        emitter.emit_freedom_fries(ProgramCycleAll(self.program_cycle_all));
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::FreedomFries(msg) => {
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
