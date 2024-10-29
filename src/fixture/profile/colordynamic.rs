//! SGM Colordynamic 575
//! The granddaddy Aquarius.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::control::prelude::*;
use crate::fixture::prelude::*;

#[derive(Debug, EmitState)]
pub struct Colordynamic {
    shutter: BoolChannelLevel<FullShutterStrobe>,
    color_rotation_on: Bool<()>,
    color_rotation_speed: UnipolarChannel,
    color_position: UnipolarChannel,
    fiber_rotation: BipolarSplitChannel,
}

impl Default for Colordynamic {
    fn default() -> Self {
        Colordynamic {
            shutter: ChannelLevel::wrap(ShutterStrobe::new(
                Bool::full_channel("ShutterOpen", 3),
                Strobe::channel("Strobe", 3, 16, 239, 255),
            )),
            color_rotation_on: Bool::new_off("ColorRotationOn", ()),
            color_rotation_speed: Unipolar::channel("ColorRotationSpeed", 1, 128, 255),
            color_position: Unipolar::channel("ColorPosition", 1, 0, 127),
            fiber_rotation: Bipolar::split_channel("FiberRotation", 2, 113, 0, 142, 255, 128)
                .with_detent(),
        }
    }
}

impl PatchAnimatedFixture for Colordynamic {
    const NAME: FixtureType = FixtureType("Colordynamic");
    fn channel_count(&self) -> usize {
        4
    }
}

impl ControllableFixture for Colordynamic {
    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        if self.shutter.control(msg, emitter)? {
            return Ok(true);
        }
        if self.color_rotation_on.control(msg, emitter)? {
            return Ok(true);
        }
        if self.color_rotation_speed.control(msg, emitter)? {
            return Ok(true);
        }
        if self.color_position.control(msg, emitter)? {
            return Ok(true);
        }
        if self.fiber_rotation.control(msg, emitter)? {
            return Ok(true);
        }
        Ok(false)
    }

    fn control_from_channel(
        &mut self,
        msg: &ChannelControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        self.shutter.control_from_channel(msg, emitter)?;
        Ok(())
    }
}

impl AnimatedFixture for Colordynamic {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        dmx_buf[0] = 0; // FIXME does this do anything?
        if self.color_rotation_on.val() {
            self.color_rotation_speed.render(
                animation_vals.filter(&AnimationTarget::ColorRotationSpeed),
                dmx_buf,
            );
        } else {
            self.color_position.render(
                animation_vals.filter(&AnimationTarget::ColorPosition),
                dmx_buf,
            );
        }
        self.fiber_rotation.render(
            animation_vals.filter(&AnimationTarget::FiberRotation),
            dmx_buf,
        );
        self.shutter
            .render_with_group(group_controls, std::iter::empty(), dmx_buf);
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
    ColorPosition,
    ColorRotationSpeed,
    FiberRotation,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        matches!(self, Self::ColorPosition | Self::ColorRotationSpeed)
    }
}
