//! Control profile for a dimmer.

use anyhow::Context;
use num_derive::{FromPrimitive, ToPrimitive};
use number::UnipolarFloat;
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use super::{
    AnimatedFixture, ControlMessagePayload, ControllableFixture, EmitFixtureStateChange,
    FixtureControlMessage, PatchAnimatedFixture,
};
use crate::{master::FixtureGroupControls, util::unipolar_to_range};

#[derive(Default, Debug)]
pub struct Dimmer(UnipolarFloat);

impl PatchAnimatedFixture for Dimmer {
    const NAME: &'static str = "dimmer";
    fn channel_count(&self) -> usize {
        1
    }
}

impl Dimmer {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
        self.0 = sc;
        emitter.emit_dimmer(sc);
    }
}

impl AnimatedFixture for Dimmer {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        _group_controls: &FixtureGroupControls,
        animation_vals: &super::animation_target::TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        let mut level = self.0.val();

        for (val, _target) in animation_vals {
            level += val;
        }
        dmx_buf[0] = unipolar_to_range(0, 255, UnipolarFloat::new(level));
    }
}

impl ControllableFixture for Dimmer {
    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        emitter.emit_dimmer(self.0);
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> anyhow::Result<()> {
        self.handle_state_change(
            *msg.unpack_as::<ControlMessage>().context(Self::NAME)?,
            emitter,
        );
        Ok(())
    }
}

pub type StateChange = UnipolarFloat;

// Venus has no controls that are not represented as state changes.
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
    Level,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        true
    }
}
