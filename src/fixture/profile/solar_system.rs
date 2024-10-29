//! Optikinetics Solar System - the grand champion gobo rotator
use crate::fixture::prelude::*;

#[derive(Debug, EmitState, Control)]
pub struct SolarSystem {
    #[channel_control]
    shutter_open: BoolChannelLevel<Bool<()>>,
    auto_shutter: Bool<()>,
    front_gobo: IndexedSelectMult,
    #[animate]
    front_rotation: Mirrored<RenderRotation>,
    rear_gobo: IndexedSelectMult,
    #[animate]
    rear_rotation: Mirrored<RenderRotation>,
}

const GOBO_COUNT: usize = 8;

impl Default for SolarSystem {
    fn default() -> Self {
        Self {
            shutter_open: Bool::new_off("ShutterOpen", ()).with_channel_level(),
            auto_shutter: Bool::new_off("AutoShutter", ()),
            front_gobo: IndexedSelect::multiple("FrontGobo", 0, false, GOBO_COUNT, 32, 16),
            front_rotation: Bipolar::new("FrontRotation", RenderRotation { dmx_buf_offset: 1 })
                .with_detent()
                .with_mirroring(true),
            rear_gobo: IndexedSelect::multiple("RearGobo", 0, false, GOBO_COUNT, 32, 16),
            rear_rotation: Bipolar::new("RearRotation", RenderRotation { dmx_buf_offset: 1 })
                .with_detent()
                .with_mirroring(true),
        }
    }
}

impl PatchAnimatedFixture for SolarSystem {
    const NAME: FixtureType = FixtureType("SolarSystem");
    fn channel_count(&self) -> usize {
        7
    }
}
impl ControllableFixture for SolarSystem {}

impl AnimatedFixture for SolarSystem {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        self.front_gobo.render_no_anim(dmx_buf);
        self.front_rotation.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::FrontRotation),
            dmx_buf,
        );
        self.rear_gobo.render_no_anim(dmx_buf);
        self.rear_gobo.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::RearRotation),
            dmx_buf,
        );
        dmx_buf[6] = if !self.shutter_open.control.val() {
            0
        } else if self.auto_shutter.val() {
            38
        } else {
            255
        };
    }
}

#[derive(Debug)]
struct RenderRotation {
    dmx_buf_offset: usize,
}

impl RenderToDmx<BipolarFloat> for RenderRotation {
    fn render(&self, val: &BipolarFloat, dmx_buf: &mut [u8]) {
        if *val == BipolarFloat::ZERO {
            dmx_buf[self.dmx_buf_offset] = 0;
            dmx_buf[self.dmx_buf_offset + 1] = 0;
            return;
        }
        dmx_buf[self.dmx_buf_offset] = if *val < BipolarFloat::ZERO { 50 } else { 77 };
        dmx_buf[self.dmx_buf_offset + 1] = unipolar_to_range(0, 255, val.abs());
    }
}
