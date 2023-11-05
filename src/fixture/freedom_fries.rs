//! Basic control profile for 8-channel auto program control of the Chauvet
//! Freedom Stick.

//! Control profle for the Chauvet Rotosphere Q3, aka Son Of Spherion.

use log::error;
use num_derive::{FromPrimitive, ToPrimitive};
use number::UnipolarFloat;

use super::{
    animation_target::TargetedAnimationValues,
    color::{Color, StateChange as ColorStateChange},
    generic::{GenericStrobe, GenericStrobeStateChange},
    AnimatedFixture, ControllableFixture, EmitFixtureStateChange, FixtureControlMessage,
    PatchAnimatedFixture,
};
use crate::{master::MasterControls, util::unipolar_to_range};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

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

impl PatchAnimatedFixture for FreedomFries {
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

impl AnimatedFixture for FreedomFries {
    type Target = AnimationTarget;
    fn render_with_animations(
        &self,
        master_controls: &MasterControls,
        animation_vals: &TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        let mut dimmer = self.dimmer.val();
        let mut speed = self.speed.val();
        for (val, target) in animation_vals {
            use AnimationTarget::*;
            match target {
                // FIXME: might want to do something nicer for unipolar values
                Dimmer => dimmer += val,
                Speed => speed += val,
            }
        }
        dmx_buf[0] = unipolar_to_range(0, 255, UnipolarFloat::new(dimmer));
        self.color
            .render_with_animations(master_controls, &[], &mut dmx_buf[1..4]);
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
        dmx_buf[7] = unipolar_to_range(0, 255, UnipolarFloat::new(speed));
    }
}

impl ControllableFixture for FreedomFries {
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

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    EnumString,
    EnumIter,
    EnumDisplay,
    FromPrimitive,
    ToPrimitive,
)]
pub enum AnimationTarget {
    #[default]
    Dimmer,
    Speed,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        matches!(self, Self::Dimmer | Self::Speed)
    }
}
