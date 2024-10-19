//! Intuitive control profile for the American DJ Aquarius 250.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Default, Debug)]
pub struct Aquarius {
    controls: GroupControlMap<ControlMessage>,
    lamp_on: bool,
    rotation: BipolarFloat,
}

impl PatchAnimatedFixture for Aquarius {
    const NAME: FixtureType = FixtureType("Aquarius");
    fn channel_count(&self) -> usize {
        2
    }
}

impl Aquarius {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        match sc {
            LampOn(v) => self.lamp_on = v,
            Rotation(v) => self.rotation = v,
        };
        Self::emit(sc, emitter);
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
        let mut rotation = self.rotation.val();
        for (val, target) in animation_vals {
            use AnimationTarget::*;
            match target {
                Rotation => rotation += val,
            }
        }
        dmx_buf[0] = bipolar_to_split_range(BipolarFloat::new(rotation), 130, 8, 132, 255, 0);
        dmx_buf[1] = if self.lamp_on { 255 } else { 0 };
    }
}

impl ControllableFixture for Aquarius {
    fn populate_controls(&mut self) {
        Self::map_controls(&mut self.controls);
    }

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        Self::emit(LampOn(self.lamp_on), emitter);
        Self::emit(Rotation(self.rotation), emitter);
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

const LAMP_ON: Button = button("LampOn");

impl Aquarius {
    fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;
        LAMP_ON.map_state(map, LampOn);
        map.add_bipolar("Rotation", |v| Rotation(bipolar_fader_with_detent(v)));
    }
}

impl HandleOscStateChange<StateChange> for Aquarius {}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    LampOn(bool),
    Rotation(BipolarFloat),
}

// Aquarius has no controls that are not represented as state changes.
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
    Rotation,
}

impl AnimationTarget {
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        false
    }
}
