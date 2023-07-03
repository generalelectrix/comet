//! Intuitive control profile for the American DJ Aquarius 250.
use num_derive::{FromPrimitive, ToPrimitive};
use number::BipolarFloat;
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use super::{
    AnimatedFixture, ControllableFixture, EmitFixtureStateChange, FixtureControlMessage, PatchAnimatedFixture, PatchFixture,
};
use crate::{master::MasterControls, util::bipolar_to_split_range};

#[derive(Default, Debug)]
pub struct Aquarius {
    lamp_on: bool,
    rotation: BipolarFloat,
}

impl PatchAnimatedFixture for Aquarius {
    const NAME: &'static str = "aquarius";
    fn channel_count(&self) -> usize {
        2
    }
}

impl Aquarius {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        match sc {
            LampOn(v) => self.lamp_on = v,
            Rotation(v) => self.rotation = v,
        };
        emitter.emit_aquarius(sc);
    }
}

impl AnimatedFixture for Aquarius {
    type Target = AnimationTarget;
    fn render_with_animations(
        &self,
        _master: &MasterControls,
        animation_vals: &super::animation_target::TargetedAnimationValues<Self::Target>,
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
    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        emitter.emit_aquarius(LampOn(self.lamp_on));
        emitter.emit_aquarius(Rotation(self.rotation));
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::Aquarius(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

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
