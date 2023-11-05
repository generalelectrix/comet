//! Flexible control profile for a single-color fixture.

use std::collections::HashMap;

use anyhow::{bail, Result};
use num_derive::{FromPrimitive, ToPrimitive};
use number::{Phase, UnipolarFloat};

use crate::master::MasterControls;

use super::{
    animation_target::TargetedAnimationValues, AnimatedFixture, ControllableFixture,
    EmitFixtureStateChange, FixtureControlMessage, PatchAnimatedFixture,
};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Default, Debug)]
pub struct Color {
    hue: Phase,
    sat: UnipolarFloat,
    val: UnipolarFloat,
    model: Model,
}

impl PatchAnimatedFixture for Color {
    const NAME: &'static str = "color";
    fn channel_count(&self) -> usize {
        self.model.channel_count()
    }

    fn new(options: &HashMap<String, String>) -> Result<Self> {
        let mut c = Self::default();
        if let Some(kind) = options.get("kind") {
            c.model = match kind.to_lowercase().as_str() {
                "rgb" => Model::Rgb,
                "rgbw" => Model::Rgbw,
                "hsv" => Model::Hsv,
                "rgbwau" => Model::Rgbwau,
                "sabre_spot" => Model::SabreSpot,
                other => {
                    bail!("unknown color model \"{}\"", other);
                }
            };
        }
        Ok(c)
    }
}

impl Color {
    pub fn handle_state_change(
        &mut self,
        sc: StateChange,
        emitter: &mut dyn EmitFixtureStateChange,
    ) {
        self.update_state(sc);
        emitter.emit_color(sc);
    }

    pub fn update_state(&mut self, sc: StateChange) {
        use StateChange::*;
        match sc {
            Hue(v) => self.hue = v,
            Sat(v) => self.sat = v,
            Val(v) => self.val = v,
        };
    }

    pub fn from_model(m: Model) -> Self {
        Self {
            model: m,
            ..Self::default()
        }
    }

    /// Call the provided callback with all controllable state.
    pub fn state<F>(&self, f: &mut F)
    where
        F: FnMut(StateChange),
    {
        use StateChange::*;
        f(Hue(self.hue));
        f(Sat(self.sat));
        f(Val(self.val));
    }
}

impl AnimatedFixture for Color {
    type Target = AnimationTarget;
    fn render_with_animations(
        &self,
        _master: &MasterControls,
        animation_vals: &TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        let mut hue = self.hue.val();
        let mut sat = self.sat.val();
        let mut val = self.val.val();
        for (anim_val, target) in animation_vals {
            use AnimationTarget::*;
            match target {
                Hue => hue += anim_val,
                // FIXME: might want to do something nicer for unipolar values
                Sat => sat += anim_val,
                Val => val += anim_val,
            }
        }
        self.model.render(
            dmx_buf,
            Phase::new(hue),
            UnipolarFloat::new(sat),
            UnipolarFloat::new(val),
        );
    }
}

impl ControllableFixture for Color {
    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        self.state(&mut |sc| emitter.emit_color(sc));
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::Color(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    Hue(Phase),
    Sat(UnipolarFloat),
    Val(UnipolarFloat),
}

// Venus has no controls that are not represented as state changes.
pub type ControlMessage = StateChange;

#[derive(Debug, Clone)]
pub enum Model {
    Rgb,
    Rgbw,
    Hsv,
    Rgbwau,
    SabreSpot,
}

impl Default for Model {
    fn default() -> Self {
        Self::Rgb
    }
}

impl Model {
    fn channel_count(&self) -> usize {
        match self {
            Self::Rgb => 3,
            Self::Rgbw => 4,
            Self::Hsv => 3,
            Self::Rgbwau => 6,
            Self::SabreSpot => 3,
        }
    }

    fn render(&self, buf: &mut [u8], hue: Phase, sat: UnipolarFloat, val: UnipolarFloat) {
        match self {
            Self::Rgb => {
                let [r, g, b] = hsv_to_rgb(hue, sat, val);
                buf[0] = r;
                buf[1] = g;
                buf[2] = b;
            }
            Self::Rgbw => {
                let rgb_slice = &mut buf[0..3];
                rgb_slice.copy_from_slice(&hsv_to_rgb(hue, sat, val));
                buf[3] = unit_to_u8((sat.invert() * val).val());
            }
            Self::Hsv => {
                buf[0] = unit_to_u8(hue.val());
                buf[1] = unit_to_u8(sat.val());
                buf[2] = unit_to_u8(val.val());
            }
            Self::Rgbwau => {
                // TODO: decide what to do with those other diodes...
                let rgb_slice = &mut buf[0..3];
                rgb_slice.copy_from_slice(&hsv_to_rgb(hue, sat, val));
            }
            Self::SabreSpot => {
                buf[0] = unit_to_u8((hue + 0.33333333333).val() * -1.0 + 1.0);
                buf[1] = unit_to_u8(sat.invert().val());
                buf[2] = unit_to_u8(val.val());
            }
        }
    }
}

type ColorRgb = [u8; 3];

fn hsv_to_rgb(hue: Phase, sat: UnipolarFloat, val: UnipolarFloat) -> ColorRgb {
    if sat == 0.0 {
        let v = unit_to_u8(val.val());
        return [v, v, v];
    }
    let var_h = if hue == 1.0 { 0.0 } else { hue.val() * 6.0 };

    let var_i = var_h.floor();
    let var_1 = val.val() * (1.0 - sat.val());
    let var_2 = val.val() * (1.0 - sat.val() * (var_h - var_i));
    let var_3 = val.val() * (1.0 - sat.val() * (1.0 - (var_h - var_i)));

    let (rv, gv, bv) = match var_i as i64 {
        0 => (val.val(), var_3, var_1),
        1 => (var_2, val.val(), var_1),
        2 => (var_1, val.val(), var_3),
        3 => (var_1, var_2, val.val()),
        4 => (var_3, var_1, val.val()),
        _ => (val.val(), var_1, var_2),
    };
    [unit_to_u8(rv), unit_to_u8(gv), unit_to_u8(bv)]
}

fn unit_to_u8(v: f64) -> u8 {
    (255. * v).round() as u8
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
    Hue,
    Sat,
    Val,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        matches!(self, Self::Sat | Self::Val)
    }
}
