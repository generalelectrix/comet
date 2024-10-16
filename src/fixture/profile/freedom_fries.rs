//! Basic control profile for 8-channel auto program control of the Chauvet
//! Freedom Stick.

//! Control profle for the Chauvet Rotosphere Q3, aka Son Of Spherion.

use anyhow::Context;
use log::error;
use num_derive::{FromPrimitive, ToPrimitive};
use number::UnipolarFloat;

use super::{
    color::{Color, StateChange as ColorStateChange},
    generic::{GenericStrobe, GenericStrobeStateChange},
};
use crate::fixture::prelude::*;
use crate::util::unipolar_to_range;
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
    const NAME: FixtureType = FixtureType("freedom_fries");
    fn channel_count(&self) -> usize {
        8
    }
}

impl FreedomFries {
    pub const PROGRAM_COUNT: usize = 27;
    fn handle_state_change(
        &mut self,
        sc: StateChange,
        emitter: &FixtureStateEmitter,
    ) {
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
        Self::emit(sc, emitter);
    }
}

impl AnimatedFixture for FreedomFries {
    type Target = AnimationTarget;
    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
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
            .render_with_animations(group_controls, &[], &mut dmx_buf[1..4]);
        dmx_buf[4] = 0;
        dmx_buf[5] = self
            .strobe
            .render_range_with_master(group_controls.strobe(), 0, 11, 255);
        dmx_buf[6] = {
            if !self.run_program {
                0
            } else if self.program_cycle_all {
                227
            } else {
                ((self.program * 8) + 11) as u8
            }
        };
        dmx_buf[7] = unipolar_to_range(0, 255, UnipolarFloat::new(speed));
    }
}

impl ControllableFixture for FreedomFries {
    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        Self::emit(Dimmer(self.dimmer), emitter);
        Self::emit(Speed(self.speed), emitter);
        Self::emit(Program(self.program), emitter);
        Self::emit(ProgramCycleAll(self.program_cycle_all), emitter);
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        self.handle_state_change(
            *msg.unpack_as::<ControlMessage>().context(Self::NAME)?,
            emitter,
        );
        Ok(())
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
