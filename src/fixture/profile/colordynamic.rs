//! SGM Colordynamic 575
//! The granddaddy Aquarius.

use crate::fixture::prelude::*;

#[derive(Debug, EmitState, Control)]
pub struct Colordynamic {
    #[channel_control]
    shutter: BoolChannelLevel<FullShutterStrobe>,
    #[animate]
    color_position: UnipolarChannel,
    color_rotation_on: Bool<()>,
    #[animate]
    color_rotation_speed: UnipolarChannel,
    #[animate]
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

impl ControllableFixture for Colordynamic {}

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
