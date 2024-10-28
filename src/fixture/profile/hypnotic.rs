//! Intuitive control profile for the American DJ Aquarius 250.
use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;
use crate::control::prelude::*;

#[derive(Debug)]
pub struct Hypnotic {
    red_laser_on: Bool<()>,
    green_laser_on: Bool<()>,
    blue_laser_on: Bool<()>,
    rotation: BipolarSplitChannelMirror,
}

impl Default for Hypnotic {
    fn default() -> Self {
        Self {
            red_laser_on: Bool::new_off("RedLaserOn", ()),
            green_laser_on: Bool::new_off("GreenLaserOn", ()),
            blue_laser_on: Bool::new_off("BlueLaserOn", ()),
            rotation: Bipolar::split_channel("Rotation", 1, 135, 245, 120, 10, 0)
                .with_detent()
                .with_mirroring(true),
        }
    }
}

impl PatchAnimatedFixture for Hypnotic {
    const NAME: FixtureType = FixtureType("Hypnotic");
    fn channel_count(&self) -> usize {
        2
    }
}

impl AnimatedFixture for Hypnotic {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        dmx_buf[0] = match (
            self.red_laser_on.val(),
            self.green_laser_on.val(),
            self.blue_laser_on.val(),
        ) {
            (false, false, false) => 0,
            (true, false, false) => 8,
            (false, true, false) => 68,
            (false, false, true) => 128,
            (true, true, false) => 38,
            (true, false, true) => 158,
            (false, true, true) => 98,
            (true, true, true) => 188,
        };
        self.rotation
            .render_with_group(group_controls, animation_vals.all(), dmx_buf);
    }
}

impl ControllableFixture for Hypnotic {
    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.red_laser_on.emit_state(emitter);
        self.green_laser_on.emit_state(emitter);
        self.blue_laser_on.emit_state(emitter);
        self.rotation.emit_state(emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        if self.red_laser_on.control(msg, emitter)? {
            return Ok(true);
        }
        if self.green_laser_on.control(msg, emitter)? {
            return Ok(true);
        }
        if self.blue_laser_on.control(msg, emitter)? {
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
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        false
    }
}
