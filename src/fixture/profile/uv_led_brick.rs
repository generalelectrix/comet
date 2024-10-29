//! Control profile for a uv_led_brick.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};


use crate::fixture::prelude::*;

#[derive(Debug, EmitState, Control)]
pub struct UvLedBrick {
    dimmer: UnipolarChannelLevel<UnipolarChannel>,
}

impl Default for UvLedBrick {
    fn default() -> Self {
        Self {
            dimmer: Unipolar::full_channel("Level", 0).with_channel_level(),
        }
    }
}

impl PatchAnimatedFixture for UvLedBrick {
    const NAME: FixtureType = FixtureType("UvLedBrick");
    fn channel_count(&self) -> usize {
        7
    }
}

impl AnimatedFixture for UvLedBrick {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        _group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        self.dimmer.render(animation_vals.all(), dmx_buf);
        dmx_buf[4] = 255;
        dmx_buf[5] = 255;
        dmx_buf[6] = 255;
    }
}

impl ControllableFixture for UvLedBrick {
    fn control_from_channel(
        &mut self,
        msg: &ChannelControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        self.dimmer.control_from_channel(msg, emitter)?;
        Ok(())
    }
}

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
