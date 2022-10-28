//! RIP the Venus, perhaps someday it will fly again.

use std::time::Duration;

use log::debug;
use number::{BipolarFloat, UnipolarFloat};

use crate::fixture::{EmitStateChange as EmitShowStateChange, StateChange as ShowStateChange};
use crate::{
    dmx::DmxAddr,
    util::{unipolar_to_range, RampingParameter},
};

/// Control abstraction for the RA venus.
/// DMX profile Venus
///
/// Motor 1 is base motor
/// Motor 2 is crescent translate motor
/// Motor 3 is saucer off axis rotate motor
/// Motor 4 is color carousel
///
/// Motor direction is split at 127
/// Lamp on/off is split at 127 (high is on)
///
/// 1 - Motor 1 Dir
/// 2 - Motor 1 Speed
/// 3 - Motor 2 Speed
/// 4 - Motor 3 Dir
/// 5 - Motor 3 Speed
/// 6 - Motor 4 Dir
/// 7 - Motor 4 Speed
/// 8 - Lamp Control
pub struct Venus {
    dmx_index: usize,
    base_rotation: RampingParameter<BipolarFloat>,
    cradle_motion: RampingParameter<UnipolarFloat>,
    head_rotation: RampingParameter<BipolarFloat>,
    color_rotation: RampingParameter<BipolarFloat>,
    lamp_on: bool,
}

impl Venus {
    pub fn new(dmx_addr: DmxAddr) -> Self {
        Self {
            dmx_index: dmx_addr - 1,
            base_rotation: RampingParameter::new(BipolarFloat::ZERO, BipolarFloat::ONE),
            cradle_motion: RampingParameter::new(UnipolarFloat::ZERO, UnipolarFloat::ONE),
            head_rotation: RampingParameter::new(BipolarFloat::ZERO, BipolarFloat::ONE),
            color_rotation: RampingParameter::new(BipolarFloat::ZERO, BipolarFloat::ONE),
            lamp_on: false,
        }
    }

    pub fn update(&mut self, delta_t: Duration) {
        self.base_rotation.update(delta_t);
        self.cradle_motion.update(delta_t);
        self.head_rotation.update(delta_t);
        self.color_rotation.update(delta_t);
    }

    /// Render into the provided DMX universe.
    pub fn render(&self, dmx_univ: &mut [u8]) {
        render_bipolar_to_dir_and_val(
            self.base_rotation.current(),
            &mut dmx_univ[self.dmx_index..self.dmx_index + 2],
        );
        dmx_univ[self.dmx_index + 2] = unipolar_to_range(0, 255, self.cradle_motion.current());
        render_bipolar_to_dir_and_val(
            self.head_rotation.current(),
            &mut dmx_univ[self.dmx_index + 3..self.dmx_index + 5],
        );
        // Limit color wheel speed to 50% (...it still chewed itself to pieces...).
        let color_wheel_scale = UnipolarFloat::new(0.5);
        render_bipolar_to_dir_and_val(
            self.color_rotation.current() * color_wheel_scale,
            &mut dmx_univ[self.dmx_index + 5..self.dmx_index + 7],
        );
        dmx_univ[7] = if self.lamp_on { 255 } else { 0 };
        debug!("{:?}", &dmx_univ[self.dmx_index..self.dmx_index + 8]);
    }

    /// Emit the current value of all controllable state.
    pub fn emit_state<E: EmitStateChange>(&self, emitter: &mut E) {
        use StateChange::*;
        emitter.emit(BaseRotation(self.base_rotation.target));
        emitter.emit(CradleMotion(self.cradle_motion.target));
        emitter.emit(HeadRotation(self.head_rotation.target));
        emitter.emit(ColorRotation(self.color_rotation.target));
        emitter.emit(LampOn(self.lamp_on));
    }

    pub fn control<E: EmitStateChange>(&mut self, msg: ControlMessage, emitter: &mut E) {
        self.handle_state_change(msg, emitter);
    }

    fn handle_state_change<E: EmitStateChange>(&mut self, sc: StateChange, emitter: &mut E) {
        use StateChange::*;
        match sc {
            BaseRotation(v) => self.base_rotation.target = v,
            CradleMotion(v) => self.cradle_motion.target = v,
            HeadRotation(v) => self.head_rotation.target = v,
            ColorRotation(v) => self.color_rotation.target = v,
            LampOn(v) => self.lamp_on = v,
        };
        emitter.emit(sc);
    }
}

fn render_bipolar_to_dir_and_val(v: BipolarFloat, out: &mut [u8]) {
    out[1] = unipolar_to_range(0, 255, v.abs());
    out[0] = if v.val() < 0.0 { 0 } else { 255 };
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    BaseRotation(BipolarFloat),
    CradleMotion(UnipolarFloat),
    HeadRotation(BipolarFloat),
    ColorRotation(BipolarFloat),
    LampOn(bool),
}

// Venus has no controls that are not represented as state changes.
pub type ControlMessage = StateChange;

pub trait EmitStateChange {
    fn emit(&mut self, sc: StateChange);
}

impl<T: EmitShowStateChange> EmitStateChange for T {
    fn emit(&mut self, sc: StateChange) {
        self.emit(ShowStateChange::Venus(sc));
    }
}
