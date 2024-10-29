//! Control profile for a dimmer.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;

#[derive(Debug, EmitState, Control)]
pub struct Dimmer {
    #[channel_control]
    level: UnipolarChannelLevel<UnipolarChannel>,
}

impl Default for Dimmer {
    fn default() -> Self {
        Self {
            level: Unipolar::full_channel("Level", 0).with_channel_level(),
        }
    }
}

impl PatchAnimatedFixture for Dimmer {
    const NAME: FixtureType = FixtureType("Dimmer");
    fn channel_count(&self) -> usize {
        1
    }
}

impl AnimatedFixture for Dimmer {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        _group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        self.level.render(animation_vals.all(), dmx_buf);
    }
}

impl ControllableFixture for Dimmer {}

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
