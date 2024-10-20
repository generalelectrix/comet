//! Intuitive control profile for the American DJ Aquarius 250.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Debug)]
pub struct Aquarius {
    lamp_on: BoolChannel,
    rotation: BipolarSplitChannel,
}

impl Default for Aquarius {
    fn default() -> Self {
        Self {
            lamp_on: Bool::full_channel("LampOn", 1),
            rotation: Bipolar::split_channel("Rotation", 0, 130, 8, 132, 255, 0),
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
        _group_controls: &FixtureGroupControls,
        animation_vals: &TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        self.rotation
            .render(animation_vals.iter().map(|(v, _)| *v), dmx_buf);
        self.lamp_on.render_no_anim(dmx_buf);
    }
}

impl ControllableFixture for Aquarius {
    fn populate_controls(&mut self) {}

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.lamp_on.emit_state(emitter);
        self.rotation.emit_state(emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        let control = msg.control();
        if control == self.lamp_on.name() {
            self.lamp_on.control(msg, emitter)?;
            return Ok(());
        }
        if control == self.rotation.name() {
            self.rotation.control(msg, emitter)?;
            return Ok(());
        }
        bail!("no control for {} matched for {control}", Self::NAME);
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
