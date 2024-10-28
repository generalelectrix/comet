//! Flexible control profile for a single-color fixture.

use std::collections::HashMap;

use anyhow::{bail, Result};
use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::control::prelude::*;
use crate::fixture::prelude::*;

#[derive(Debug)]
pub struct Color {
    hue: PhaseControl<()>,
    sat: Unipolar<()>,
    val: Unipolar<()>,
    model: Model,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            hue: PhaseControl::new("Hue", ()),
            sat: Unipolar::new("Sat", ()),
            val: Unipolar::new("Val", ()),
            model: Default::default(),
        }
    }
}

impl PatchAnimatedFixture for Color {
    const NAME: FixtureType = FixtureType("Color");
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
    pub fn from_model(m: Model) -> Self {
        Self {
            model: m,
            ..Self::default()
        }
    }

    pub fn render_without_animations(&self, dmx_buf: &mut [u8]) {
        self.model
            .render(dmx_buf, self.hue.val(), self.sat.val(), self.val.val());
    }
}

impl AnimatedFixture for Color {
    type Target = AnimationTarget;
    fn render_with_animations(
        &self,
        _group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        let mut hue = self.hue.val().val();
        let mut sat = self.sat.val().val();
        let mut val = self.val.val().val();
        for (anim_val, target) in animation_vals.iter() {
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
    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        OscControl::emit_state(self, emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        OscControl::control(self, msg, emitter)
    }
}

impl OscControl<()> for Color {
    fn control_direct(
        &mut self,
        _val: (),
        _emitter: &dyn crate::osc::EmitScopedOscMessage,
    ) -> anyhow::Result<()> {
        bail!("direct control is not implemented for Color controls");
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &dyn crate::osc::EmitScopedOscMessage,
    ) -> anyhow::Result<bool> {
        if self.hue.control(msg, emitter)? {
            return Ok(true);
        }
        if self.sat.control(msg, emitter)? {
            return Ok(true);
        }
        if self.val.control(msg, emitter)? {
            return Ok(true);
        }
        Ok(false)
    }

    fn emit_state(&self, emitter: &dyn crate::osc::EmitScopedOscMessage) {
        self.hue.emit_state(emitter);
        self.sat.emit_state(emitter);
        self.val.emit_state(emitter);
    }
}

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

    pub fn render(&self, buf: &mut [u8], hue: Phase, sat: UnipolarFloat, val: UnipolarFloat) {
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
