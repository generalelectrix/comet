//! Control profile for the "house light" Starlight white laser moonflower.

use anyhow::Context;
use num_derive::{FromPrimitive, ToPrimitive};
use number::{BipolarFloat, UnipolarFloat};

use super::generic::{GenericStrobe, GenericStrobeStateChange};
use super::{
    AnimatedFixture, ControllableFixture, EmitFixtureStateChange as EmitShowStateChange,
    FixtureControlMessage, PatchAnimatedFixture,
};
use crate::master::FixtureGroupControls;
use crate::util::bipolar_to_split_range;
use crate::util::unipolar_to_range;
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Default, Debug)]
pub struct Starlight {
    dimmer: UnipolarFloat,
    strobe: GenericStrobe,
    rotation: BipolarFloat,
}

impl PatchAnimatedFixture for Starlight {
    const NAME: &'static str = "starlight";
    fn channel_count(&self) -> usize {
        4
    }
}

impl Starlight {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitShowStateChange) {
        use StateChange::*;
        match sc {
            Dimmer(v) => self.dimmer = v,
            Rotation(v) => self.rotation = v,
            Strobe(v) => self.strobe.handle_state_change(v),
        };
        emitter.emit_starlight(sc);
    }
}

impl AnimatedFixture for Starlight {
    type Target = AnimationTarget;
    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: &super::animation_target::TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        dmx_buf[0] = 255; // DMX mode
        let mut dimmer = self.dimmer.val();
        let mut rotation = self.rotation.val();
        for (val, target) in animation_vals {
            use AnimationTarget::*;
            match target {
                // FIXME: might want to do something nicer for unipolar values
                Rotation => rotation += val,
                Dimmer => dimmer += val,
            }
        }
        dmx_buf[1] = unipolar_to_range(0, 255, UnipolarFloat::new(dimmer));
        dmx_buf[2] = self
            .strobe
            .render_range_with_master(group_controls.strobe(), 0, 10, 255);
        dmx_buf[3] = bipolar_to_split_range(BipolarFloat::new(rotation), 127, 1, 128, 255, 0);
    }
}

impl ControllableFixture for Starlight {
    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitShowStateChange,
    ) -> anyhow::Result<()> {
        self.handle_state_change(
            *msg.unpack_as::<ControlMessage>().context(Self::NAME)?,
            emitter,
        );
        Ok(())
    }

    fn emit_state(&self, emitter: &mut dyn EmitShowStateChange) {
        use StateChange::*;
        emitter.emit_starlight(Dimmer(self.dimmer));
        emitter.emit_starlight(Rotation(self.rotation));
        let mut emit_strobe = |ssc| {
            emitter.emit_starlight(Strobe(ssc));
        };
        self.strobe.emit_state(&mut emit_strobe);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    Dimmer(UnipolarFloat),
    Strobe(GenericStrobeStateChange),
    Rotation(BipolarFloat),
}

// Starlight has no controls that are not represented as state changes.
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
