//! Control profle for the Chauvet Rotosphere Q3, aka Son Of Spherion.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use super::color::Model::Rgbw;

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Debug)]
pub struct RotosphereQ3 {
    hue: PhaseControl<()>,
    sat: Unipolar<()>,
    val: UnipolarChannelLevel<Unipolar<()>>,
    strobe: StrobeChannel,
    rotation: BipolarSplitChannel,
}

impl Default for RotosphereQ3 {
    fn default() -> Self {
        Self {
            hue: PhaseControl::new("Phase", ()),
            sat: Unipolar::new("Sat", ()),
            val: Unipolar::new("Val", ()).with_channel_level(),
            strobe: Strobe::channel("Strobe", 4, 1, 250, 0),
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

impl ControllableFixture for RotosphereQ3 {
    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.hue.emit_state(emitter);
        self.sat.emit_state(emitter);
        self.val.emit_state(emitter);
        self.strobe.emit_state(emitter);
        self.rotation.emit_state(emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        if self.hue.control(msg, emitter)? {
            return Ok(true);
        }
        if self.sat.control(msg, emitter)? {
            return Ok(true);
        }
        if self.val.control(msg, emitter)? {
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

    fn control_from_channel(
        &mut self,
        msg: &ChannelControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        self.val.control_from_channel(msg, emitter)?;
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
