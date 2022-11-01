//! Intuitive control profile for the American DJ H2O DMX Pro.

use log::debug;
use number::{BipolarFloat, UnipolarFloat};

use crate::fixture::{
    ControlMessage as ShowControlMessage, EmitStateChange as EmitShowStateChange, Fixture,
};
use crate::util::bipolar_to_split_range;
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

    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitShowStateChange) {
        use StateChange::*;
        match sc {
            Dimmer(v) => self.dimmer = v,
            Rotation(v) => self.rotation = v,
            FixedColor(v) => self.fixed_color = v,
            ColorRotate(v) => self.color_rotate = v,
            ColorRotation(v) => self.color_rotation = v,
        };
        emitter.emit_h2o(sc);
    }
}

impl Fixture for H2O {
    fn render(&self, dmx_univ: &mut [u8]) {
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

    fn control(
        &mut self,
        msg: ShowControlMessage,
        emitter: &mut dyn EmitShowStateChange,
    ) -> Option<ShowControlMessage> {
        match msg {
            ShowControlMessage::H2O(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }

    fn emit_state(&self, emitter: &mut dyn EmitShowStateChange) {
        use StateChange::*;
        emitter.emit_h2o(Dimmer(self.dimmer));
        emitter.emit_h2o(Rotation(self.rotation));
        emitter.emit_h2o(FixedColor(self.fixed_color));
        emitter.emit_h2o(ColorRotate(self.color_rotate));
        emitter.emit_h2o(ColorRotation(self.color_rotation));
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
