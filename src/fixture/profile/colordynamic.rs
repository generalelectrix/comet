//! SGM Colordynamic 575
//! The granddaddy Aquarius.

use num_derive::{FromPrimitive, ToPrimitive};
use number::{BipolarFloat, UnipolarFloat};

use super::generic::{GenericStrobe, GenericStrobeStateChange};
use crate::fixture::prelude::*;
use crate::master::FixtureGroupControls;
use crate::util::{bipolar_to_split_range, unipolar_to_range};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Default, Debug)]
pub struct Colordynamic {
    controls: GroupControlMap<ControlMessage>,
    shutter_open: bool,
    strobe: GenericStrobe,
    color_rotation_on: bool,
    color_rotation_speed: UnipolarFloat,
    color_position: UnipolarFloat,
    fiber_rotation: BipolarFloat,
}

// impl Default for Colordynamic {
//     fn default() -> Self {
//         Colordynamic {
//             shutter_open: true,
//             strobe: GenericStrobe::default(),
//             color_rotation_on: true,
//             color_rotation_speed: UnipolarFloat::new(0.1),
//             color_position: UnipolarFloat::ZERO,
//             fiber_rotation: BipolarFloat::new(0.1),
//         }
//     }
// }

impl PatchAnimatedFixture for Colordynamic {
    const NAME: FixtureType = FixtureType("Colordynamic");
    fn channel_count(&self) -> usize {
        4
    }
}

impl Colordynamic {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        match sc {
            ShutterOpen(v) => self.shutter_open = v,
            Strobe(sc) => self.strobe.handle_state_change(sc),
            ColorRotationSpeed(v) => self.color_rotation_speed = v,
            ColorPosition(v) => self.color_position = v,
            FiberRotation(v) => self.fiber_rotation = v,
            ColorRotationOn(v) => self.color_rotation_on = v,
        };
        Self::emit(sc, emitter);
    }
}

impl ControllableFixture for Colordynamic {
    fn populate_controls(&mut self) {
        Self::map_controls(&mut self.controls);
    }

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        Self::emit(ShutterOpen(self.shutter_open), emitter);
        let mut emit_strobe = |ssc| {
            Self::emit(Strobe(ssc), emitter);
        };
        self.strobe.emit_state(&mut emit_strobe);
        Self::emit(ColorRotationOn(self.color_rotation_on), emitter);
        Self::emit(ColorRotationSpeed(self.color_rotation_speed), emitter);
        Self::emit(ColorPosition(self.color_position), emitter);
        Self::emit(FiberRotation(self.fiber_rotation), emitter);
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

impl AnimatedFixture for Colordynamic {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
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
            unipolar_to_range(128, 255, UnipolarFloat::new(color_rotation_speed))
        } else {
            unipolar_to_range(0, 127, UnipolarFloat::new(color_position))
        };
        dmx_buf[2] =
            bipolar_to_split_range(BipolarFloat::new(fiber_rotation), 113, 0, 142, 255, 128);
        dmx_buf[3] = if !self.shutter_open {
            0
        } else {
            let strobe_off = 0;
            let strobe =
                self.strobe
                    .render_range_with_master(group_controls.strobe(), strobe_off, 16, 239);
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
