//! Flexible control profile for a single-color fixture.

use std::{collections::HashMap, error::Error};

use number::{Phase, UnipolarFloat};
use simple_error::bail;

use crate::master::MasterControls;

use super::{EmitFixtureStateChange, Fixture, FixtureControlMessage, PatchFixture};

#[derive(Default, Debug)]
pub struct Color {
    hue: Phase,
    sat: UnipolarFloat,
    val: UnipolarFloat,
    model: Model,
}

impl PatchFixture for Color {
    fn channel_count(&self) -> usize {
        self.model.channel_count()
    }

    fn new(options: &HashMap<String, String>) -> Result<Self, Box<dyn Error>> {
        let mut c = Self::default();
        if let Some(kind) = options.get("kind") {
            c.model = match kind.to_lowercase().as_str() {
                "rgb" => Model::rgb(),
                "rgbw" => Model::rgbw(),
                "hsv" => Model::hsv(),
                "rgbwau" => Model::rgbwau(),
                "sabre_spot" => Model::sabre_spot(),
                other => {
                    bail!("unknown color model \"{}\"", other);
                }
            };
        }
        Ok(c)
    }
}

impl Color {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        match sc {
            Hue(v) => self.hue = v,
            Sat(v) => self.sat = v,
            Val(v) => self.val = v,
        };
        self.model.update(self.hue, self.sat, self.val);
        emitter.emit_color(sc);
    }
}

impl Fixture for Color {
    fn render(&self, _master_controls: &MasterControls, dmx_buf: &mut [u8]) {
        dmx_buf.copy_from_slice(self.model.vals());
    }

    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        emitter.emit_color(Hue(self.hue));
        emitter.emit_color(Sat(self.sat));
        emitter.emit_color(Val(self.val));
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

#[derive(Debug)]
enum Model {
    Rgb([u8; 3]),
    Rgbw([u8; 4]),
    Hsv([u8; 3]),
    Rgbwau([u8; 6]),
    SabreSpot([u8; 3]),
}

impl Default for Model {
    fn default() -> Self {
        Self::rgb()
    }
}

impl Model {
    fn rgb() -> Self {
        Self::Rgb([0; 3])
    }

    fn hsv() -> Self {
        Self::Hsv([0; 3])
    }

    fn rgbw() -> Self {
        Self::Rgbw([0; 4])
    }

    fn rgbwau() -> Self {
        Self::Rgbwau([0; 6])
    }

    fn sabre_spot() -> Self {
        Self::SabreSpot([0; 3])
    }

    fn channel_count(&self) -> usize {
        match self {
            Self::Rgb(_) => 3,
            Self::Rgbw(_) => 4,
            Self::Hsv(_) => 3,
            Self::Rgbwau(_) => 6,
            Self::SabreSpot(_) => 3,
        }
    }

    fn update(&mut self, hue: Phase, sat: UnipolarFloat, val: UnipolarFloat) {
        match self {
            Self::Rgb(vals) => {
                *vals = hsv_to_rgb(hue, sat, val);
            }
            Self::Rgbw(vals) => {
                // TODO: decide what to do with white
                let rgb_slice = &mut vals[0..3];
                rgb_slice.copy_from_slice(&hsv_to_rgb(hue, sat, val));
            }
            Self::Hsv(vals) => {
                vals[0] = unit_to_u8(hue.val());
                vals[1] = unit_to_u8(sat.val());
                vals[2] = unit_to_u8(val.val());
            }
            Self::Rgbwau(vals) => {
                // TODO: decide what to do with those other diodes...
                let rgb_slice = &mut vals[0..3];
                rgb_slice.copy_from_slice(&hsv_to_rgb(hue, sat, val));
            }
            Self::SabreSpot(vals) => {
                vals[0] = unit_to_u8((hue + 0.33333333333).val() * -1.0 + 1.0);
                vals[1] = unit_to_u8(sat.invert().val());
                vals[2] = unit_to_u8(val.val());
            }
        }
    }

    fn vals(&self) -> &[u8] {
        match self {
            Self::Rgb(v) => v,
            Self::Rgbw(v) => v,
            Self::Hsv(v) => v,
            Self::Rgbwau(v) => v,
            Self::SabreSpot(v) => v,
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
