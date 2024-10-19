//! Control profile for a uv_led_brick.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Default, Debug)]
pub struct UvLedBrick(UnipolarFloat, GroupControlMap<ControlMessage>);

impl PatchAnimatedFixture for UvLedBrick {
    const NAME: FixtureType = FixtureType("UvLedBrick");
    fn channel_count(&self) -> usize {
        7
    }
}

impl UvLedBrick {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &FixtureStateEmitter) {
        self.0 = sc;
        Self::emit(sc, emitter);
    }
}

impl AnimatedFixture for UvLedBrick {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        _group_controls: &FixtureGroupControls,
        animation_vals: &TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        let mut level = self.0.val();

        for (val, _target) in animation_vals {
            level += val;
        }
        dmx_buf[0] = unipolar_to_range(0, 255, UnipolarFloat::new(level));
        dmx_buf[4] = 255;
        dmx_buf[5] = 255;
        dmx_buf[6] = 255;
    }
}

impl ControllableFixture for UvLedBrick {
    fn populate_controls(&mut self) {
        Self::map_controls(&mut self.1);
    }

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        Self::emit(self.0, emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        let Some((ctl, _)) = self.1.handle(msg)? else {
            return Ok(());
        };
        self.handle_state_change(ctl, emitter);
        Ok(())
    }
}

pub type StateChange = UnipolarFloat;

// Venus has no controls that are not represented as state changes.
pub type ControlMessage = StateChange;

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

impl UvLedBrick {
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        map.add_unipolar("Level", |x| x);
    }
}

impl HandleOscStateChange<StateChange> for UvLedBrick {}
