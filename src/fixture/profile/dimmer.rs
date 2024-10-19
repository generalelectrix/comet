//! Control profile for a dimmer.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Default, Debug)]
pub struct Dimmer {
    level: UnipolarFloat,
    controls: GroupControlMap<ControlMessage>,
}

impl PatchAnimatedFixture for Dimmer {
    const NAME: FixtureType = FixtureType("Dimmer");
    fn channel_count(&self) -> usize {
        1
    }
}

impl Dimmer {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &FixtureStateEmitter) {
        self.level = sc;
        Self::emit(sc, emitter);
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
        let mut level = self.level.val();

        for (val, _target) in animation_vals {
            level += val;
        }
        dmx_buf[0] = unipolar_to_range(0, 255, UnipolarFloat::new(level));
    }
}

impl ControllableFixture for Dimmer {
    fn populate_controls(&mut self) {
        Self::map_controls(&mut self.controls);
    }

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        Self::emit(self.level, emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        let Some((ctl, _)) = self.controls.handle(msg)? else {
            return Ok(());
        };
        self.handle_state_change(ctl, emitter);
        Ok(())
    }
}

pub type StateChange = UnipolarFloat;

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

impl Dimmer {
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        map.add_unipolar("Level", |x| x);
    }
}

impl HandleOscStateChange<StateChange> for Dimmer {
    fn emit_osc_state_change<S>(_sc: StateChange, _send: &S)
    where
        S: crate::osc::EmitScopedOscMessage + ?Sized,
    {
    }
}
