//! Optikinetics Solar System - the grand champion gobo rotator

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Debug)]
pub struct SolarSystem {
    shutter_open: BoolChannelLevel<Bool<()>>,
    auto_shutter: Bool<()>,
    front_gobo: IndexedSelectMult,
    front_rotation: Mirrored<RenderRotation>,
    rear_gobo: IndexedSelectMult,
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
                .with_mirroring(true),
            rear_gobo: IndexedSelect::multiple("RearGobo", 0, false, GOBO_COUNT, 32, 16),
            rear_rotation: Bipolar::new("RearRotation", RenderRotation { dmx_buf_offset: 1 })
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
impl ControllableFixture for SolarSystem {
    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.shutter_open.emit_state(emitter);
        self.auto_shutter.emit_state(emitter);
        self.front_gobo.emit_state(emitter);
        self.front_rotation.emit_state(emitter);
        self.rear_gobo.emit_state(emitter);
        self.rear_rotation.emit_state(emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        if self.shutter_open.control(msg, emitter)? {
            return Ok(true);
        }
        if self.auto_shutter.control(msg, emitter)? {
            return Ok(true);
        }
        if self.front_gobo.control(msg, emitter)? {
            return Ok(true);
        }
        if self.front_rotation.control(msg, emitter)? {
            return Ok(true);
        }
        if self.rear_gobo.control(msg, emitter)? {
            return Ok(true);
        }
        if self.rear_rotation.control(msg, emitter)? {
            return Ok(true);
        }
        Ok(false)
    }

    fn control_from_channel(
        &mut self,
        msg: &ChannelControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        self.shutter_open.control_from_channel(msg, emitter)?;
        Ok(())
    }
}

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
    FrontRotation,
    RearRotation,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        false
    }
}
