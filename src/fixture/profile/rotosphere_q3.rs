//! Control profle for the Chauvet Rotosphere Q3, aka Son Of Spherion.
use super::color::Model::Rgbw;

use crate::fixture::prelude::*;

#[derive(Debug, EmitState, Control)]
pub struct RotosphereQ3 {
    #[animate]
    rotation: BipolarSplitChannel,
    #[animate]
    hue: PhaseControl<()>,
    #[animate]
    sat: Unipolar<()>,
    #[channel_control]
    #[animate]
    val: ChannelLevelUnipolar<Unipolar<()>>,
    strobe: StrobeChannel,
}

impl Default for RotosphereQ3 {
    fn default() -> Self {
        Self {
            hue: PhaseControl::new("Phase", ()),
            sat: Unipolar::new("Sat", ()).at_full(),
            val: Unipolar::new("Val", ()).with_channel_level(),
            strobe: Strobe::channel("Strobe", 4, 1, 250, 0),
            rotation: Bipolar::split_channel("Rotation", 5, 1, 127, 129, 255, 0).with_detent(),
        }
    }
}

impl PatchAnimatedFixture for RotosphereQ3 {
    const NAME: FixtureType = FixtureType("RotosphereQ3");
    fn channel_count(&self) -> usize {
        9
    }
}

impl AnimatedFixture for RotosphereQ3 {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        Rgbw.render(
            &mut dmx_buf[0..4],
            self.hue
                .val_with_anim(animation_vals.filter(&AnimationTarget::Hue)),
            self.sat
                .val_with_anim(animation_vals.filter(&AnimationTarget::Sat)),
            self.val
                .control
                .val_with_anim(animation_vals.filter(&AnimationTarget::Val)),
        );
        self.strobe
            .render_with_group(group_controls, std::iter::empty(), dmx_buf);
        self.rotation
            .render(animation_vals.filter(&AnimationTarget::Rotation), dmx_buf);
        dmx_buf[6] = 0;
        dmx_buf[7] = 0;
        dmx_buf[8] = 0;
    }
}

impl ControllableFixture for RotosphereQ3 {}
