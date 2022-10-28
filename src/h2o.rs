//! Intuitive control profile for the American DJ H2O DMX Pro.

use std::time::Duration;

use number::{BipolarFloat, UnipolarFloat};

use crate::{dmx::DmxAddr, util::unipolar_float_to_range};

pub struct H2O {
    dmx_index: usize,
    dimmer: UnipolarFloat,
    rotation: BipolarFloat,
    fixed_color: FixedColor,
    color_rotate: bool,
    color_rotation: BipolarFloat,
}

impl H2O {
    pub fn new(dmx_addr: DmxAddr) -> Self {
        Self {
            dmx_index: dmx_addr - 1,
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
        let dmx_slice = &mut dmx_univ[self.dmx_index..self.dmx_index + 3];
        dmx_slice[0] = unipolar_float_to_range(0, 255, self.dimmer);
        dmx_slice[1] = render_bipolar_to_split_range(self.rotation, 120, 10, 135, 245, 0);
    }
}

fn render_bipolar_to_split_range(
    v: BipolarFloat,
    cw_slow: u8,
    cw_fast: u8,
    ccw_slow: u8,
    ccw_fast: u8,
    stop: u8,
) -> u8 {
    unimplemented!()
}

#[derive(Copy, Clone, Debug)]
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
