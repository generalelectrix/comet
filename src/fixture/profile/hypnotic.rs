//! Intuitive control profile for the American DJ Aquarius 250.

use anyhow::Context;
use number::BipolarFloat;


use crate::fixture::prelude::*;
use crate::util::bipolar_to_split_range;
use num_derive::{FromPrimitive, ToPrimitive};

use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Default, Debug)]
pub struct Hypnotic {
    red_laser_on: bool,
    green_laser_on: bool,
    blue_laser_on: bool,
    rotation: BipolarFloat,
}

impl PatchAnimatedFixture for Hypnotic {
    const NAME: FixtureType = FixtureType("hypnotic");
    fn channel_count(&self) -> usize {
        2
    }
}

impl Hypnotic {
    fn handle_state_change(
        &mut self,
        sc: StateChange,
        emitter: &mut dyn crate::osc::EmitControlMessage,
    ) {
        use StateChange::*;
        match sc {
            RedLaserOn(v) => self.red_laser_on = v,
            GreenLaserOn(v) => self.green_laser_on = v,
            BlueLaserOn(v) => self.blue_laser_on = v,
            Rotation(v) => self.rotation = v,
        };
        Self::emit(sc, emitter);
    }
}

impl AnimatedFixture for Hypnotic {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        _group_controls: &FixtureGroupControls,
        animation_vals: &TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        dmx_buf[0] = match (self.red_laser_on, self.green_laser_on, self.blue_laser_on) {
            (false, false, false) => 0,
            (true, false, false) => 8,
            (false, true, false) => 68,
            (false, false, true) => 128,
            (true, true, false) => 38,
            (true, false, true) => 158,
            (false, true, true) => 98,
            (true, true, true) => 188,
        };
        let mut rotation = self.rotation;
        for (val, target) in animation_vals {
            match target {
                AnimationTarget::Rotation => rotation += *val,
            }
        }
        dmx_buf[1] = bipolar_to_split_range(self.rotation, 135, 245, 120, 10, 0);
    }
}

impl ControllableFixture for Hypnotic {
    fn emit_state(&self, emitter: &mut dyn crate::osc::EmitControlMessage) {
        use StateChange::*;
        Self::emit(RedLaserOn(self.red_laser_on), emitter);
        Self::emit(GreenLaserOn(self.green_laser_on), emitter);
        Self::emit(BlueLaserOn(self.blue_laser_on), emitter);
        Self::emit(Rotation(self.rotation), emitter);
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn crate::osc::EmitControlMessage,
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
    RedLaserOn(bool),
    GreenLaserOn(bool),
    BlueLaserOn(bool),
    Rotation(BipolarFloat),
}

// Hypnotic has no controls that are not represented as state changes.
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
    Rotation,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        false
    }
}
