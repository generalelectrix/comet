//! Intuitive control profile for the American DJ H2O DMX Pro.

use number::{BipolarFloat, UnipolarFloat};

use super::{
    EmitFixtureStateChange as EmitShowStateChange, Fixture, FixtureControlMessage, PatchFixture,
};
use crate::util::bipolar_to_split_range;
use crate::util::unipolar_to_range;
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Default, Debug)]
pub struct H2O {
    dimmer: UnipolarFloat,
    rotation: BipolarFloat,
    fixed_color: FixedColor,
    color_rotate: bool,
    color_rotation: BipolarFloat,
}

impl PatchFixture for H2O {
    const CHANNEL_COUNT: usize = 3;
}

impl H2O {
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
    fn render(&self, dmx_buf: &mut [u8]) {
        dmx_buf[0] = unipolar_to_range(0, 255, self.dimmer);
        dmx_buf[1] = bipolar_to_split_range(self.rotation, 120, 10, 135, 245, 0);
        if self.color_rotate {
            dmx_buf[2] = bipolar_to_split_range(self.color_rotation, 186, 128, 197, 255, 187);
        } else {
            dmx_buf[2] = self.fixed_color.as_dmx();
        }
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitShowStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::H2O(msg) => {
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

#[derive(Default, Copy, Clone, Debug, PartialEq, EnumString, EnumIter, EnumDisplay)]
pub enum FixedColor {
    #[default]
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
