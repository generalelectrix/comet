//! Control profle for the Chauvet Rotosphere Q3, aka Son Of Spherion.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use super::color::{AnimationTarget as ColorAnimationTarget, Color, Model as ColorModel};
use super::strobe::{Strobe, StrobeChannel};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Debug)]
pub struct RotosphereQ3 {
    color: Color,
    strobe: StrobeChannel,
    rotation: BipolarSplitChannel,
}

impl Default for RotosphereQ3 {
    fn default() -> Self {
        Self {
            color: Color::from_model(ColorModel::Rgbw),
            strobe: Strobe::full_channel("Strobe", 4, 1, 250, 0),
            rotation: Bipolar::split_channel("Rotation", 5, 1, 127, 129, 255, 0),
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
        let mut color_anim_vals = vec![];
        for (val, target) in animation_vals.iter() {
            use AnimationTarget::*;
            match target {
                // FIXME: would really like to avoid allocating here for nested
                // animation target case.
                Hue => color_anim_vals.push((*val, ColorAnimationTarget::Hue)),
                Sat => color_anim_vals.push((*val, ColorAnimationTarget::Sat)),
                Val => color_anim_vals.push((*val, ColorAnimationTarget::Val)),
                _ => (),
            }
        }
        self.color.render_with_animations(
            group_controls,
            TargetedAnimationValues(&color_anim_vals),
            &mut dmx_buf[0..4],
        );
        self.strobe
            .render_with_master(group_controls.strobe(), dmx_buf);
        self.rotation
            .render(animation_vals.filter(&AnimationTarget::Rotation), dmx_buf);
        dmx_buf[6] = 0;
        dmx_buf[7] = 0;
        dmx_buf[8] = 0;
    }
}

impl ControllableFixture for RotosphereQ3 {
    fn populate_controls(&mut self) {}

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        OscControl::emit_state(&self.color, emitter);
        self.strobe.emit_state(emitter);
        self.rotation.emit_state(emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        if OscControl::control(&mut self.color, msg, emitter)? {
            return Ok(true);
        }
        if self.strobe.control(msg, emitter)? {
            return Ok(true);
        }
        if self.rotation.control(msg, emitter)? {
            return Ok(true);
        }
        Ok(false)
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
    Rotation,
    Hue,
    Sat,
    Val,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        false
    }
}
