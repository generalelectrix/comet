use std::time::Duration;

use log::debug;
use number::{BipolarFloat, UnipolarFloat};

use crate::fixture::{ControlMessage as ShowControlMessage, EmitStateChange, Fixture};
use crate::generic::{GenericStrobe, GenericStrobeStateChange};
use crate::{
    dmx::DmxAddr,
    util::{unipolar_to_range, RampingParameter},
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
    dmx_index: usize,
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
            dmx_index: dmx_addr - 1,
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

    fn render_ball_rotation(&self, dmx_slice: &mut [u8]) {
        let val = self.ball_rotation.current().val();
        let mut speed = val.abs();
        let direction = val >= 0.;
        if self.ball_start && speed < 0.2 {
            speed = 0.2;
        }
        let dmx_speed = unipolar_to_range(0, MAX_ROTATION_SPEED, UnipolarFloat::new(speed));
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
        unipolar_to_range(0, 255, speed)
    }

    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitStateChange) {
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
        emitter.emit_lumasphere(sc);
    }
}

impl Fixture for Lumasphere {
    fn update(&mut self, delta_t: Duration) {
        self.ball_rotation.update(delta_t);
    }

    fn render(&self, dmx_univ: &mut [u8]) {
        self.render_ball_rotation(&mut dmx_univ[self.dmx_index..self.dmx_index + 2]);
        dmx_univ[self.dmx_index + 2] = self.render_color_rotation();
        self.strobe_1
            .render(&mut dmx_univ[self.dmx_index + 3..self.dmx_index + 5]);
        self.strobe_2
            .render(&mut dmx_univ[self.dmx_index + 5..self.dmx_index + 7]);
        dmx_univ[self.dmx_index + 7] = unipolar_to_range(0, 255, self.lamp_1_intensity);
        dmx_univ[self.dmx_index + 8] = unipolar_to_range(0, 255, self.lamp_2_intensity);
        debug!("{:?}", &dmx_univ[self.dmx_index..self.dmx_index + 9]);
    }

    fn emit_state(&self, emitter: &mut dyn EmitStateChange) {
        use StateChange::*;
        emitter.emit_lumasphere(Lamp1Intensity(self.lamp_1_intensity));
        emitter.emit_lumasphere(Lamp2Intensity(self.lamp_2_intensity));
        emitter.emit_lumasphere(BallRotation(self.ball_rotation.current()));
        emitter.emit_lumasphere(BallStart(self.ball_start));
        emitter.emit_lumasphere(ColorRotation(self.color_rotation));
        emitter.emit_lumasphere(ColorStart(self.color_start));
        self.strobe_1.emit_state(emitter, Strobe1);
        self.strobe_2.emit_state(emitter, Strobe2);
    }

    fn control(
        &mut self,
        msg: ShowControlMessage,
        emitter: &mut dyn EmitStateChange,
    ) -> Option<ShowControlMessage> {
        match msg {
            ShowControlMessage::Lumasphere(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

#[derive(Default)]
pub struct Strobe {
    state: GenericStrobe,
    intensity: UnipolarFloat,
}

impl Strobe {
    fn render(&self, dmx_slice: &mut [u8]) {
        let (intensity, rate) = if self.state.on() {
            (
                unipolar_to_range(0, 255, self.intensity),
                unipolar_to_range(0, 255, self.state.rate()),
            )
        } else {
            (0, 0)
        };
        dmx_slice[0] = intensity;
        dmx_slice[1] = rate;
    }

    fn emit_state<F>(&self, emitter: &mut dyn EmitStateChange, wrap: F)
    where
        F: Fn(StrobeStateChange) -> StateChange + 'static,
    {
        use StrobeStateChange::*;
        emitter.emit_lumasphere(wrap(Intensity(self.intensity)));
        let mut emit = |ssc| {
            emitter.emit_lumasphere(wrap(State(ssc)));
        };
        self.state.emit_state(&mut emit);
    }

    fn handle_state_change(&mut self, sc: StrobeStateChange) {
        use StrobeStateChange::*;
        match sc {
            State(v) => self.state.handle_state_change(v),
            Intensity(v) => self.intensity = v,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StrobeStateChange {
    Intensity(UnipolarFloat),
    State(GenericStrobeStateChange),
}

#[derive(Clone, Copy, Debug)]
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
