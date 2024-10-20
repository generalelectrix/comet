//! Control profile for a dimmer.

use anyhow::bail;
use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Debug)]
pub struct Dimmer {
    level: Unipolar<RenderUnipolarToRange>,
}

impl Default for Dimmer {
    fn default() -> Self {
        Self {
            level: Unipolar::full_channel("Level", 0),
        }
    }
}

impl PatchAnimatedFixture for Dimmer {
    const NAME: FixtureType = FixtureType("Dimmer");
    fn channel_count(&self) -> usize {
        1
    }
}

impl AnimatedFixture for Dimmer {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        _group_controls: &FixtureGroupControls,
        animation_vals: &TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        self.level
            .render(animation_vals.iter().map(|(v, _)| *v), dmx_buf);
    }
}

impl ControllableFixture for Dimmer {
    fn populate_controls(&mut self) {}

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.level.emit_state(emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        let control = msg.control();
        if control == self.level.name() {
            self.level.control(msg, emitter)?;
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
    Level,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        true
    }
}
