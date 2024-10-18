//! Clay Paky Astroscan - drunken sailor extraordinaire

use log::error;
use num_derive::{FromPrimitive, ToPrimitive};
use number::{BipolarFloat, UnipolarFloat};

use super::generic::{GenericStrobe, GenericStrobeStateChange};
use crate::fixture::prelude::*;

use crate::util::{bipolar_to_range, bipolar_to_split_range, unipolar_to_range};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Debug)]
struct Active(bool);

impl Default for Active {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Default, Debug)]
pub struct Astroscan {
    controls: GroupControlMap<ControlMessage>,
    lamp_on: bool,
    dimmer: UnipolarFloat,
    strobe: GenericStrobe,
    iris: UnipolarFloat,
    color: Color,
    gobo: usize,
    gobo_rotation: BipolarFloat,
    mirror_rotation: BipolarFloat,
    pan: BipolarFloat,
    tilt: BipolarFloat,
    mirror: Mirror,
    active: Active,
}

impl PatchAnimatedFixture for Astroscan {
    const NAME: FixtureType = FixtureType("Astroscan");
    fn channel_count(&self) -> usize {
        9
    }
}

impl Astroscan {
    pub const GOBO_COUNT: usize = 5; // includes the open position

    fn handle_state_change(&mut self, sc: StateChange, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        match sc {
            LampOn(v) => self.lamp_on = v,
            Dimmer(v) => self.dimmer = v,
            Strobe(sc) => self.strobe.handle_state_change(sc),
            Color(c) => self.color = c,
            Iris(v) => self.iris = v,
            Gobo(v) => {
                if v >= Self::GOBO_COUNT {
                    error!("Gobo select index {} out of range.", v);
                    return;
                }
                self.gobo = v;
            }
            GoboRotation(v) => self.gobo_rotation = v,
            MirrorGoboRotation(v) => self.mirror.gobo_rotation = v,
            MirrorRotation(v) => self.mirror_rotation = v,
            MirrorMirrorRotation(v) => self.mirror.mirror_rotation = v,
            Pan(v) => self.pan = v,
            MirrorPan(v) => self.mirror.pan = v,
            Tilt(v) => self.tilt = v,
            MirrorTilt(v) => self.mirror.tilt = v,
            Active(v) => self.active.0 = v,
        };
        Self::emit(sc, emitter);
        Self::emit(sc, emitter);
    }
}

impl ControllableFixture for Astroscan {
    fn populate_controls(&mut self) {
        Self::map_controls(&mut self.controls);
    }

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        Self::emit(LampOn(self.lamp_on), emitter);
        Self::emit(Dimmer(self.dimmer), emitter);
        let mut emit_strobe = |ssc| {
            Self::emit(Strobe(ssc), emitter);
        };
        self.strobe.emit_state(&mut emit_strobe);
        Self::emit(Color(self.color), emitter);
        Self::emit(Iris(self.iris), emitter);
        Self::emit(Gobo(self.gobo), emitter);
        Self::emit(GoboRotation(self.gobo_rotation), emitter);
        Self::emit(MirrorGoboRotation(self.mirror.gobo_rotation), emitter);
        Self::emit(MirrorRotation(self.mirror_rotation), emitter);
        Self::emit(MirrorMirrorRotation(self.mirror.mirror_rotation), emitter);
        Self::emit(Pan(self.pan), emitter);
        Self::emit(MirrorPan(self.mirror.pan), emitter);
        Self::emit(Tilt(self.tilt), emitter);
        Self::emit(MirrorTilt(self.mirror.tilt), emitter);
        Self::emit(Active(self.active.0), emitter);
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

impl AnimatedFixture for Astroscan {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: &TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        if !self.active.0 {
            dmx_buf.fill(0);
            return;
        }
        let mut dimmer = self.dimmer.val();
        let mut iris = self.iris.val();
        let mut gobo_rotation = self.gobo_rotation.val();
        let mut mirror_rotation = self.mirror_rotation.val();
        let mut pan = self.pan.val();
        let mut tilt = self.tilt.val();
        for (val, target) in animation_vals {
            use AnimationTarget::*;
            match target {
                Dimmer => dimmer += val,
                Iris => iris += val,
                GoboRotation => gobo_rotation += val,
                MirrorRotation => mirror_rotation += val,
                Pan => pan += val,
                Tilt => tilt += val,
            }
        }
        dmx_buf[0] = unipolar_to_range(0, 255, UnipolarFloat::new(iris));
        dmx_buf[1] = self.color.as_dmx();
        dmx_buf[2] = if self.lamp_on { 255 } else { 0 };
        dmx_buf[3] = {
            let strobe_off = 0;
            let strobe =
                self.strobe
                    .render_range_with_master(group_controls.strobe(), strobe_off, 140, 243);
            if strobe == strobe_off {
                unipolar_to_range(0, 139, UnipolarFloat::new(dimmer))
            } else {
                strobe
            }
        };
        dmx_buf[4] = bipolar_to_range(
            0,
            255,
            BipolarFloat::new(pan).invert_if(group_controls.mirror),
        );
        dmx_buf[5] = bipolar_to_range(0, 255, BipolarFloat::new(tilt));
        dmx_buf[6] = self.gobo as u8 * 55;
        dmx_buf[7] = bipolar_to_split_range(
            BipolarFloat::new(gobo_rotation).invert_if(group_controls.mirror),
            189,
            128,
            193,
            255,
            191,
        );
        dmx_buf[8] = bipolar_to_split_range(
            BipolarFloat::new(mirror_rotation).invert_if(group_controls.mirror),
            189,
            128,
            193,
            255,
            191,
        );
    }
}

#[derive(Debug)]
struct Mirror {
    mirror_rotation: bool,
    gobo_rotation: bool,
    pan: bool,
    tilt: bool,
}

impl Default for Mirror {
    fn default() -> Self {
        Self {
            mirror_rotation: true,
            gobo_rotation: true,
            pan: true,
            tilt: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    LampOn(bool),
    Dimmer(UnipolarFloat),
    Strobe(GenericStrobeStateChange),
    Color(Color),
    Iris(UnipolarFloat),
    Gobo(usize),
    GoboRotation(BipolarFloat),
    MirrorRotation(BipolarFloat),
    Pan(BipolarFloat),
    Tilt(BipolarFloat),
    MirrorGoboRotation(bool),
    MirrorMirrorRotation(bool),
    MirrorPan(bool),
    MirrorTilt(bool),
    Active(bool),
}

pub type ControlMessage = StateChange;

#[derive(Copy, Clone, Debug, Default, PartialEq, EnumString, EnumIter, EnumDisplay)]
pub enum Color {
    #[default]
    Open,
    Red,
    Yellow,
    Violet,
    Green,
    Orange,
    Blue,
    Pink,
}

impl Color {
    fn as_dmx(self) -> u8 {
        use Color::*;
        match self {
            Open => 0,
            Red => 14,
            Yellow => 32,
            Violet => 51,
            Green => 67,
            Orange => 81,
            Blue => 98,
            Pink => 115, // 127 back to white
        }
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
    Dimmer,
    Iris,
    GoboRotation,
    MirrorRotation,
    Pan,
    Tilt,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        matches!(self, Self::Dimmer | Self::Iris)
    }
}
