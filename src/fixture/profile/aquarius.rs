//! Intuitive control profile for the American DJ Aquarius 250.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Debug)]
pub struct Aquarius {
    lamp_on: BoolChannelLevel<BoolChannel>,
    rotation: BipolarSplitChannel,
}

impl Default for Aquarius {
    fn default() -> Self {
        Self {
            lamp_on: Bool::full_channel("LampOn", 1).with_channel_level(),
            rotation: Bipolar::split_channel("Rotation", 0, 130, 8, 132, 255, 0).with_detent(),
        }
    }
}

impl PatchAnimatedFixture for Aquarius {
    const NAME: FixtureType = FixtureType("Aquarius");
    fn channel_count(&self) -> usize {
        2
    }
}

impl AnimatedFixture for Aquarius {
    type Target = AnimationTarget;
    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        self.rotation
            .render_with_group(group_controls, animation_vals.all(), dmx_buf);
        self.lamp_on
            .render_with_group(group_controls, std::iter::empty(), dmx_buf);
    }
}

impl ControllableFixture for Aquarius {
    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.lamp_on.emit_state(emitter);
        self.rotation.emit_state(emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        if self.lamp_on.control(msg, emitter)? {
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
        self.lamp_on.control_from_channel(msg, emitter)?;
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
}

impl AnimationTarget {
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        false
    }
}
