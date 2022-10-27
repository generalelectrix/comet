use std::time::Duration;

use log::debug;
use number::{BipolarFloat, UnipolarFloat};

use crate::fixture::{EmitStateChange as EmitShowStateChange, StateChange as ShowStateChange};
use crate::{
    dmx::{self, DmxAddr},
    util::{unit_float_to_range, RampingParameter},
};

/// DMX 255 is too fast; restrict to a reasonable value.
const MAX_ROTATION_SPEED: u8 = 100;

/// Control abstraction for the lumapshere.
///
/// lumasphere DMX profile:
///
/// 1: outer ball rotation speed
/// note: requires a value of ~17% in order to be activated
/// (ball start button)
///
/// 2: outer ball rotation direction
/// split halfway
///
/// 3: color wheel rotation
/// (might want to implement bump start)
///
/// 4: strobe 1 intensity
/// 5: strobe 1 rate
/// 6: strobe 2 intensity
/// 7: strobe 2 rate
///
/// There are also two lamp dimmer channels, which are conventionally set to be
/// the two channels after the lumasphere's built-in controller:
/// 8: lamp 1 dimmer
/// 9: lamp 2 dimmer
pub struct Lumasphere {
    dmx_addr: DmxAddr,
    lamp_1_intensity: UnipolarFloat,
    lamp_2_intensity: UnipolarFloat,
    ball_rotation: RampingParameter<BipolarFloat>,
    ball_start: bool,
    color_rotation: UnipolarFloat,
    color_start: bool,
    strobe_1: Strobe,
    strobe_2: Strobe,
}

impl Lumasphere {
    pub fn new(dmx_addr: DmxAddr) -> Self {
        Self {
            dmx_addr,
            lamp_1_intensity: UnipolarFloat::ZERO,
            lamp_2_intensity: UnipolarFloat::ZERO,
            // Ramp ball rotation no faster than unit range in one second.
            ball_rotation: RampingParameter::new(BipolarFloat::ZERO, BipolarFloat::ONE),
            ball_start: false,
            color_rotation: UnipolarFloat::ZERO,
            color_start: false,
            strobe_1: Strobe::default(),
            strobe_2: Strobe::default(),
        }
    }

    pub fn update(&mut self, delta_t: Duration) {
        self.ball_rotation.update(delta_t);
    }

    fn render_ball_rotation(&self, dmx_slice: &mut [u8]) {
        let val = self.ball_rotation.current().val();
        let mut speed = val.abs();
        let direction = val >= 0.;
        if self.ball_start && speed < 0.2 {
            speed = 0.2;
        }
        let dmx_speed = unit_float_to_range(0, MAX_ROTATION_SPEED, UnipolarFloat::new(speed));
        let dmx_direction = if direction { 0 } else { 255 };
        dmx_slice[0] = dmx_speed;
        dmx_slice[1] = dmx_direction;
    }

    fn render_color_rotation(&self) -> u8 {
        let speed = if self.color_start && self.color_rotation.val() < 0.2 {
            UnipolarFloat::new(0.2)
        } else {
            self.color_rotation
        };
        unit_float_to_range(0, 255, speed)
    }

    /// Render into the provided DMX universe.
    pub fn render(&self, dmx_univ: &mut [u8]) {
        self.render_ball_rotation(&mut dmx_univ[self.dmx_addr..self.dmx_addr + 2]);
        dmx_univ[self.dmx_addr + 2] = self.render_color_rotation();
        self.strobe_1
            .render(&mut dmx_univ[self.dmx_addr + 3..self.dmx_addr + 5]);
        self.strobe_2
            .render(&mut dmx_univ[self.dmx_addr + 5..self.dmx_addr + 7]);
        dmx_univ[self.dmx_addr + 7] = unit_float_to_range(0, 255, self.lamp_1_intensity);
        dmx_univ[self.dmx_addr + 8] = unit_float_to_range(0, 255, self.lamp_2_intensity);
        debug!("{:?}", &dmx_univ[self.dmx_addr..self.dmx_addr + 9]);
    }

    /// Emit the current value of all controllable state.
    pub fn emit_state<E: EmitStateChange>(&self, emitter: &mut E) {
        use StateChange::*;
        emitter.emit(Lamp1Intensity(self.lamp_1_intensity));
        emitter.emit(Lamp2Intensity(self.lamp_2_intensity));
        emitter.emit(BallRotation(self.ball_rotation.current()));
        emitter.emit(BallStart(self.ball_start));
        emitter.emit(ColorRotation(self.color_rotation));
        emitter.emit(ColorStart(self.color_start));
        self.strobe_1.emit_state(emitter, Strobe1);
        self.strobe_2.emit_state(emitter, Strobe2);
    }

    pub fn control<E: EmitStateChange>(&mut self, msg: ControlMessage, emitter: &mut E) {
        self.handle_state_change(msg, emitter);
    }

    fn handle_state_change<E: EmitStateChange>(&mut self, sc: StateChange, emitter: &mut E) {
        use StateChange::*;
        match sc {
            Lamp1Intensity(v) => self.lamp_1_intensity = v,
            Lamp2Intensity(v) => self.lamp_2_intensity = v,
            BallRotation(v) => self.ball_rotation.target = v,
            BallStart(v) => self.ball_start = v,
            ColorRotation(v) => self.color_rotation = v,
            ColorStart(v) => self.color_start = v,
            Strobe1(sc) => self.strobe_1.handle_state_change(sc),

            Strobe2(sc) => self.strobe_2.handle_state_change(sc),
        };
        emitter.emit(sc);
    }
}

pub struct Strobe {
    on: bool,
    intensity: UnipolarFloat,
    rate: UnipolarFloat,
}

impl Default for Strobe {
    fn default() -> Self {
        Self {
            on: false,
            intensity: UnipolarFloat::ZERO,
            rate: UnipolarFloat::ZERO,
        }
    }
}

impl Strobe {
    fn render(&self, dmx_slice: &mut [u8]) {
        let (intensity, rate) = if self.on {
            (
                unit_float_to_range(0, 255, self.intensity),
                unit_float_to_range(0, 255, self.rate),
            )
        } else {
            (0, 0)
        };
        dmx_slice[0] = intensity;
        dmx_slice[1] = rate;
    }

    fn emit_state<E: EmitStateChange, F>(&self, emitter: &mut E, wrap: F)
    where
        F: Fn(StrobeStateChange) -> StateChange + 'static,
    {
        use StrobeStateChange::*;
        emitter.emit(wrap(On(self.on)));
        emitter.emit(wrap(Intensity(self.intensity)));
        emitter.emit(wrap(Rate(self.rate)));
    }

    fn handle_state_change(&mut self, sc: StrobeStateChange) {
        use StrobeStateChange::*;
        match sc {
            On(v) => self.on = v,
            Intensity(v) => self.intensity = v,
            Rate(v) => self.rate = v,
        }
    }
}

#[derive(Clone, Copy)]
pub enum StrobeStateChange {
    On(bool),
    Intensity(UnipolarFloat),
    Rate(UnipolarFloat),
}

#[derive(Clone, Copy)]
pub enum StateChange {
    Lamp1Intensity(UnipolarFloat),
    Lamp2Intensity(UnipolarFloat),
    BallRotation(BipolarFloat),
    BallStart(bool),
    ColorRotation(UnipolarFloat),
    ColorStart(bool),
    Strobe1(StrobeStateChange),
    Strobe2(StrobeStateChange),
}

// Lumasphere has no controls that are not represented as state changes.
pub type ControlMessage = StateChange;

pub trait EmitStateChange {
    fn emit(&mut self, sc: StateChange);
}

impl<T: EmitShowStateChange> EmitStateChange for T {
    fn emit(&mut self, sc: StateChange) {
        self.emit(ShowStateChange::Lumasphere(sc));
    }
}
