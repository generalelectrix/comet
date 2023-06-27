//! SGM Colordynamic 575
//! The granddaddy Aquarius.

use num_derive::{FromPrimitive, ToPrimitive};
use number::{BipolarFloat, UnipolarFloat};

use super::animation_target::TargetedAnimationValues;
use super::generic::{GenericStrobe, GenericStrobeStateChange};
use super::{
    AnimatedFixture, ControllableFixture, EmitFixtureStateChange, FixtureControlMessage,
    PatchAnimatedFixture,
};
use crate::master::MasterControls;
use crate::util::{bipolar_to_range, unipolar_to_range};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Default, Debug)]
pub struct Colordynamic {
    shutter_open: bool,
    strobe: GenericStrobe,
    color_rotation_on: bool,
    color_rotation_speed: UnipolarFloat,
    color_position: UnipolarFloat,
    fiber_rotation: BipolarFloat,
}

impl PatchAnimatedFixture for Colordynamic {
    const NAME: &'static str = "colordynamic";
    fn channel_count(&self) -> usize {
        4
    }
}

impl Colordynamic {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        match sc {
            ShutterOpen(v) => self.shutter_open = v,
            Strobe(sc) => self.strobe.handle_state_change(sc),
            ColorRotationSpeed(v) => self.color_rotation_speed = v,
            ColorPosition(v) => self.color_position = v,
            FiberRotation(v) => self.fiber_rotation = v,
            ColorRotationOn(v) => self.color_rotation_on = v,
        };
        emitter.emit_colordynamic(sc);
    }
}

impl ControllableFixture for Colordynamic {
    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        emitter.emit_colordynamic(ShutterOpen(self.shutter_open));
        let mut emit_strobe = |ssc| {
            emitter.emit_colordynamic(Strobe(ssc));
        };
        self.strobe.emit_state(&mut emit_strobe);
        emitter.emit_colordynamic(ColorRotationOn(self.color_rotation_on));
        emitter.emit_colordynamic(ColorRotationSpeed(self.color_rotation_speed));
        emitter.emit_colordynamic(ColorPosition(self.color_position));
        emitter.emit_colordynamic(FiberRotation(self.fiber_rotation));
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::Colordynamic(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

impl AnimatedFixture for Colordynamic {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        master: &MasterControls,
        animation_vals: &TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        let mut color_rotation_speed = self.color_rotation_speed.val();
        let mut color_position = self.color_position.val();
        let mut fiber_rotation = self.fiber_rotation.val();
        for (val, target) in animation_vals {
            use AnimationTarget::*;
            match target {
                // FIXME: might want to do something nicer for unipolar values
                ColorRotationSpeed => color_rotation_speed += val,
                ColorPosition => color_position += val,
                FiberRotation => fiber_rotation += val,
            }
        }
        dmx_buf[0] = 0; // FIXME does this do anything?
        dmx_buf[1] = if self.color_rotation_on {
            unipolar_to_range(128, 255, self.color_rotation_speed)
        } else {
            unipolar_to_range(0, 127, self.color_position)
        };
        dmx_buf[2] = bipolar_to_range(0, 255, self.fiber_rotation);
        dmx_buf[3] = if !self.shutter_open {
            0
        } else {
            let strobe_off = 0;
            let strobe = self
                .strobe
                .render_range_with_master(master.strobe(), strobe_off, 10, 240);
            if strobe == strobe_off {
                255
            } else {
                strobe
            }
        };
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    ShutterOpen(bool),
    Strobe(GenericStrobeStateChange),
    ColorRotationSpeed(UnipolarFloat),
    ColorPosition(UnipolarFloat),
    FiberRotation(BipolarFloat),
    ColorRotationOn(bool),
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
    ColorPosition,
    ColorRotationSpeed,
    FiberRotation,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        matches!(self, Self::ColorPosition | Self::ColorRotationSpeed)
    }
}
