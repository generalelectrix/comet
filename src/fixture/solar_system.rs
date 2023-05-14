//! Optikinetics Solar System - the grand champion gobo rotator

use log::{debug, error};
use num_derive::{FromPrimitive, ToPrimitive};
use number::{BipolarFloat, UnipolarFloat};

use super::animation_target::TargetedAnimationValues;
use super::{
    AnimatedFixture, ControllableFixture, EmitFixtureStateChange, FixtureControlMessage,
    PatchAnimatedFixture,
};
use crate::master::MasterControls;
use crate::util::{bipolar_to_range, bipolar_to_split_range, unipolar_to_range};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Default, Debug)]
pub struct SolarSystem {
    shutter_open: bool,
    auto_shutter: bool,
    front_gobo: usize,
    front_rotation: BipolarFloat,
    rear_gobo: usize,
    rear_rotation: BipolarFloat,
}

impl PatchAnimatedFixture for SolarSystem {
    const NAME: &'static str = "solar_system";
    fn channel_count(&self) -> usize {
        7
    }
}

impl SolarSystem {
    pub const GOBO_COUNT: usize = 8;

    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        match sc {
            ShutterOpen(v) => self.shutter_open = v,
            AutoShutter(v) => self.auto_shutter = v,
            FrontGobo(v) => {
                if v >= Self::GOBO_COUNT {
                    error!("Gobo select index {} out of range.", v);
                    return;
                }
                self.front_gobo = v;
            }
            FrontRotation(v) => self.front_rotation = v,
            RearGobo(v) => {
                if v >= Self::GOBO_COUNT {
                    error!("Gobo select index {} out of range.", v);
                    return;
                }
                self.rear_gobo = v;
            }
            RearRotation(v) => self.rear_rotation = v,
        };
        emitter.emit_solar_system(sc);
    }
}

impl ControllableFixture for SolarSystem {
    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        emitter.emit_solar_system(ShutterOpen(self.shutter_open));
        emitter.emit_solar_system(AutoShutter(self.auto_shutter));
        emitter.emit_solar_system(FrontGobo(self.front_gobo));
        emitter.emit_solar_system(FrontRotation(self.front_rotation));
        emitter.emit_solar_system(RearGobo(self.rear_gobo));
        emitter.emit_solar_system(RearRotation(self.rear_rotation));
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::SolarSystem(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

impl AnimatedFixture for SolarSystem {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        _master: &MasterControls,
        animation_vals: &TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        let mut front_rotation = self.front_rotation.val();
        let mut rear_rotation = self.rear_rotation.val();
        for (val, target) in animation_vals {
            use AnimationTarget::*;
            match target {
                FrontRotation => front_rotation += val,
                RearRotation => rear_rotation += val,
            }
        }
        dmx_buf[0] = render_gobo_select(self.front_gobo);
        render_rotation(BipolarFloat::new(front_rotation), &mut dmx_buf[1..3]);
        dmx_buf[3] = render_gobo_select(self.rear_gobo);
        render_rotation(BipolarFloat::new(rear_rotation), &mut dmx_buf[4..6]);
        dmx_buf[6] = if !self.shutter_open {
            0
        } else if self.auto_shutter {
            38
        } else {
            255
        };
    }
}

fn render_gobo_select(v: usize) -> u8 {
    (v * 32 + 16) as u8
}

fn render_rotation(v: BipolarFloat, mode_speed_buf: &mut [u8]) {
    if v == BipolarFloat::ZERO {
        mode_speed_buf[0] = 0;
        mode_speed_buf[1] = 0;
        return;
    }
    mode_speed_buf[0] = if v < BipolarFloat::ZERO { 50 } else { 77 };
    mode_speed_buf[1] = unipolar_to_range(0, 255, v.abs());
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    ShutterOpen(bool),
    AutoShutter(bool),
    FrontGobo(usize),
    FrontRotation(BipolarFloat),
    RearGobo(usize),
    RearRotation(BipolarFloat),
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
    FrontRotation,
    RearRotation,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        false
    }
}
