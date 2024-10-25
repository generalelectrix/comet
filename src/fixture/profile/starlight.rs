//! Control profile for the "house light" Starlight white laser moonflower.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Debug)]
pub struct Starlight {
    dimmer: UnipolarChannelLevel<UnipolarChannel>,
    strobe: StrobeChannel,
    rotation: BipolarSplitChannelMirror,
}
impl Default for Starlight {
    fn default() -> Self {
        Self {
            dimmer: Unipolar::full_channel("Dimmer", 1).with_channel_level(),
            strobe: Strobe::channel("Strobe", 2, 10, 255, 0),
            rotation: Bipolar::split_channel("Rotation", 3, 127, 1, 128, 255, 0)
                .with_detent()
                .with_mirroring(true),
        }
    }
}

impl PatchAnimatedFixture for Starlight {
    const NAME: FixtureType = FixtureType("Starlight");
    fn channel_count(&self) -> usize {
        4
    }
}

impl AnimatedFixture for Starlight {
    type Target = AnimationTarget;
    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        dmx_buf[0] = 255; // DMX mode
        self.dimmer
            .render(animation_vals.filter(&AnimationTarget::Dimmer), dmx_buf);
        self.strobe
            .render_with_group(group_controls, std::iter::empty(), dmx_buf);
        self.rotation.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::Rotation),
            dmx_buf,
        );
    }
}

impl ControllableFixture for Starlight {
    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        if self.dimmer.control(msg, emitter)? {
            return Ok(true);
        }
        if self.strobe.control(msg, emitter)? {
            return Ok(true);
        }
        if self.rotation.control(msg, emitter)? {
            return Ok(true);
        }
        Ok(true)
    }

    fn control_from_channel(
        &mut self,
        msg: &ChannelControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        self.dimmer.control_from_channel(msg, emitter)?;
        Ok(())
    }

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.dimmer.emit_state(emitter);
        self.strobe.emit_state(emitter);
        self.rotation.emit_state(emitter);
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
    Dimmer,
    Rotation,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        matches!(self, Self::Dimmer)
    }
}
