//! Basic control profile for 8-channel auto program control of the Chauvet
//! Freedom Stick.

//! Control profle for the Chauvet Rotosphere Q3, aka Son Of Spherion.

use log::error;
use number::UnipolarFloat;

use super::{
    color::{Color, StateChange as ColorStateChange},
    generic::{GenericStrobe, GenericStrobeStateChange},
    EmitFixtureStateChange, Fixture, FixtureControlMessage, PatchFixture,
};
use crate::{master::MasterControls, util::unipolar_to_range};

#[derive(Default, Debug)]
pub struct FreedomFries {
    dimmer: UnipolarFloat,
    color: Color,
    speed: UnipolarFloat,
    strobe: GenericStrobe,
    run_program: bool,
    program: usize,
    program_cycle_all: bool,
}

impl PatchFixture for FreedomFries {
    const NAME: &'static str = "freedom_fries";
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
            Color(v) => self.color.update_state(v),
            Strobe(sc) => self.strobe.handle_state_change(sc),
            Speed(v) => self.speed = v,
            RunProgram(v) => self.run_program = v,
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
    fn render(&self, master_controls: &MasterControls, dmx_buf: &mut [u8]) {
        dmx_buf[0] = unipolar_to_range(0, 255, self.dimmer);
        self.color.render(master_controls, &mut dmx_buf[1..4]);
        dmx_buf[4] = 0;
        dmx_buf[5] = self
            .strobe
            .render_range_with_master(master_controls.strobe(), 0, 11, 255);
        dmx_buf[6] = {
            let autopilot = master_controls.autopilot();
            let program = if autopilot.on() {
                autopilot.program() % Self::PROGRAM_COUNT
            } else {
                self.program
            };
            if !self.run_program {
                0
            } else if !autopilot.on() && self.program_cycle_all {
                227
            } else {
                ((program * 8) + 11) as u8
            }
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
    Color(ColorStateChange),
    Strobe(GenericStrobeStateChange),
    RunProgram(bool),
    Speed(UnipolarFloat),
    Program(usize),
    ProgramCycleAll(bool),
}

pub type ControlMessage = StateChange;
