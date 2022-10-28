//! Intuitive control profile for the American DJ H2O DMX Pro.

use std::time::Duration;

use log::debug;
use number::{BipolarFloat, UnipolarFloat};

use crate::fixture::{EmitStateChange as EmitShowStateChange, StateChange as ShowStateChange};
use crate::{dmx::DmxAddr, util::unipolar_to_range};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

/// Aggregate and control one or more H2Os.
pub struct H2O {
    /// Addresses of the H2Os under control.
    dmx_indices: Vec<usize>,
    dimmer: UnipolarFloat,
    rotation: BipolarFloat,
    fixed_color: FixedColor,
    color_rotate: bool,
    color_rotation: BipolarFloat,
}

impl H2O {
    pub fn new(dmx_addrs: Vec<DmxAddr>) -> Self {
        Self {
            dmx_indices: dmx_addrs.iter().map(|a| a - 1).collect(),
            dimmer: UnipolarFloat::ZERO,
            rotation: BipolarFloat::ZERO,
            fixed_color: FixedColor::White,
            color_rotate: false,
            color_rotation: BipolarFloat::ZERO,
        }
    }

    pub fn update(&mut self, _: Duration) {}

    /// Render into the provided DMX universe.
    pub fn render(&self, dmx_univ: &mut [u8]) {
        for dmx_index in self.dmx_indices.iter() {
            let dmx_slice = &mut dmx_univ[*dmx_index..*dmx_index + 3];
            dmx_slice[0] = unipolar_to_range(0, 255, self.dimmer);
            dmx_slice[1] = bipolar_to_split_range(self.rotation, 120, 10, 135, 245, 0);
            if self.color_rotate {
                dmx_slice[2] = bipolar_to_split_range(self.color_rotation, 186, 128, 197, 255, 187);
            } else {
                dmx_slice[2] = self.fixed_color.as_dmx();
            }
            debug!("{:?}", dmx_slice);
        }
    }

    /// Emit the current value of all controllable state.
    pub fn emit_state<E: EmitStateChange>(&self, emitter: &mut E) {
        use StateChange::*;
        emitter.emit(Dimmer(self.dimmer));
        emitter.emit(Rotation(self.rotation));
        emitter.emit(FixedColor(self.fixed_color));
        emitter.emit(ColorRotate(self.color_rotate));
        emitter.emit(ColorRotation(self.color_rotation));
    }

    pub fn control<E: EmitStateChange>(&mut self, msg: ControlMessage, emitter: &mut E) {
        self.handle_state_change(msg, emitter);
    }

    fn handle_state_change<E: EmitStateChange>(&mut self, sc: StateChange, emitter: &mut E) {
        use StateChange::*;
        match sc {
            Dimmer(v) => self.dimmer = v,
            Rotation(v) => self.rotation = v,
            FixedColor(v) => self.fixed_color = v,
            ColorRotate(v) => self.color_rotate = v,
            ColorRotation(v) => self.color_rotation = v,
        };
        emitter.emit(sc);
    }
}

fn bipolar_to_split_range(
    v: BipolarFloat,
    cw_slow: u8,
    cw_fast: u8,
    ccw_slow: u8,
    ccw_fast: u8,
    stop: u8,
) -> u8 {
    if v == BipolarFloat::ZERO {
        stop
    } else if v.val() > 0.0 {
        unipolar_to_range(cw_slow, cw_fast, v.abs())
    } else {
        unipolar_to_range(ccw_slow, ccw_fast, v.abs())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, EnumString, EnumIter, EnumDisplay)]
pub enum FixedColor {
    White,
    WhiteOrange,
    Orange,
    OrangeGreen,
    Green,
    GreenBlue,
    Blue,
    BlueYellow,
    Yellow,
    YellowPurple,
    Purple,
    PurpleWhite,
}

impl FixedColor {
    fn as_dmx(self) -> u8 {
        use FixedColor::*;
        match self {
            White => 0,
            WhiteOrange => 11,
            Orange => 22,
            OrangeGreen => 33,
            Green => 44,
            GreenBlue => 55,
            Blue => 66,
            BlueYellow => 77,
            Yellow => 88,
            YellowPurple => 99,
            Purple => 110,
            PurpleWhite => 121,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    Dimmer(UnipolarFloat),
    Rotation(BipolarFloat),
    FixedColor(FixedColor),
    ColorRotate(bool),
    ColorRotation(BipolarFloat),
}

// H2O has no controls that are not represented as state changes.
pub type ControlMessage = StateChange;

pub trait EmitStateChange {
    fn emit(&mut self, sc: StateChange);
}

impl<T: EmitShowStateChange> EmitStateChange for T {
    fn emit(&mut self, sc: StateChange) {
        self.emit(ShowStateChange::H2O(sc));
    }
}
