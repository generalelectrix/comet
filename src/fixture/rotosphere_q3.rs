//! Control profle for the Chauvet Rotosphere Q3, aka Son Of Spherion.

use num_derive::{FromPrimitive, ToPrimitive};
use number::BipolarFloat;
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use super::animation_target::TargetedAnimationValues;
use super::color::{Color, Model as ColorModel, StateChange as ColorStateChange};
use super::generic::{GenericStrobe, GenericStrobeStateChange};
use super::{
    AnimatedFixture, ControllableFixture, EmitFixtureStateChange, FixtureControlMessage,
    NonAnimatedFixture, PatchAnimatedFixture,
};
use crate::master::MasterControls;
use crate::util::bipolar_to_split_range;

#[derive(Debug)]
pub struct RotosphereQ3 {
    color: Color,
    strobe: GenericStrobe,
    rotation: BipolarFloat,
}

impl Default for RotosphereQ3 {
    fn default() -> Self {
        Self {
            color: Color::from_model(ColorModel::rgbw()),
            strobe: GenericStrobe::default(),
            rotation: BipolarFloat::default(),
        }
    }
}

impl PatchAnimatedFixture for RotosphereQ3 {
    const NAME: &'static str = "rotosphere_q3";
    fn channel_count(&self) -> usize {
        9
    }
}

impl RotosphereQ3 {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        match sc {
            Color(c) => self.color.update_state(c),
            Strobe(sc) => self.strobe.handle_state_change(sc),
            Rotation(v) => self.rotation = v,
        };
        emitter.emit_rotosphere_q3(sc);
    }
}

impl AnimatedFixture for RotosphereQ3 {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        master: &MasterControls,
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
        self.color.render(master, &mut dmx_buf[0..4]);
        dmx_buf[4] = self
            .strobe
            .render_range_with_master(master.strobe(), 0, 1, 250);
        dmx_buf[5] = bipolar_to_split_range(BipolarFloat::new(rotation), 1, 127, 129, 255, 0);
        dmx_buf[6] = 0;
        dmx_buf[7] = 0;
        dmx_buf[8] = 0;
    }
}

impl ControllableFixture for RotosphereQ3 {
    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        let mut emit_color = |sc| {
            emitter.emit_rotosphere_q3(Color(sc));
        };
        self.color.state(&mut emit_color);
        let mut emit_strobe = |ssc| {
            emitter.emit_rotosphere_q3(Strobe(ssc));
        };
        self.strobe.emit_state(&mut emit_strobe);
        emitter.emit_rotosphere_q3(Rotation(self.rotation));
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::RotosphereQ3(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    Color(ColorStateChange),
    Strobe(GenericStrobeStateChange),
    Rotation(BipolarFloat),
}

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
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        false
    }
}
