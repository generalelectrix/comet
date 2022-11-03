//! Control profle for the Chauvet Rotosphere Q3, aka Son Of Spherion.

use log::debug;
use number::{BipolarFloat, UnipolarFloat};

use crate::dmx::DmxAddr;
use crate::fixture::{ControlMessage as ShowControlMessage, EmitStateChange, Fixture};
use crate::generic::{GenericStrobe, GenericStrobeStateChange};
use crate::util::{bipolar_to_split_range, unipolar_to_range};

pub struct RotosphereQ3 {
    dmx_index: usize,
    red: UnipolarFloat,
    green: UnipolarFloat,
    blue: UnipolarFloat,
    white: UnipolarFloat,
    strobe: GenericStrobe,
    rotation: BipolarFloat,
}

impl RotosphereQ3 {
    const CHANNEL_COUNT: usize = 9;
    pub fn new(dmx_addr: DmxAddr) -> Self {
        Self {
            dmx_index: dmx_addr - 1,
            red: UnipolarFloat::ZERO,
            green: UnipolarFloat::ZERO,
            blue: UnipolarFloat::ZERO,
            white: UnipolarFloat::ZERO,
            strobe: GenericStrobe::default(),
            rotation: BipolarFloat::ZERO,
        }
    }

    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitStateChange) {
        use StateChange::*;
        match sc {
            Red(v) => self.red = v,
            Green(v) => self.green = v,
            Blue(v) => self.blue = v,
            White(v) => self.white = v,
            Strobe(sc) => self.strobe.handle_state_change(sc),
            Rotation(v) => self.rotation = v,
        };
        emitter.emit_rotosphere_q3(sc);
    }
}

impl Fixture for RotosphereQ3 {
    fn render(&self, dmx_univ: &mut [u8]) {
        let dmx_slice = &mut dmx_univ[self.dmx_index..self.dmx_index + Self::CHANNEL_COUNT];
        dmx_slice[0] = unipolar_to_range(0, 255, self.red);
        dmx_slice[1] = unipolar_to_range(0, 255, self.green);
        dmx_slice[2] = unipolar_to_range(0, 255, self.blue);
        dmx_slice[3] = unipolar_to_range(0, 255, self.white);
        dmx_slice[4] = if self.strobe.on() {
            unipolar_to_range(1, 250, self.strobe.rate())
        } else {
            0
        };
        dmx_slice[5] = bipolar_to_split_range(self.rotation, 1, 127, 129, 255, 0);
        dmx_slice[6] = 0; // TODO auto programs
        dmx_slice[7] = 0; // TODO auto program speed
        dmx_slice[8] = 0; // TODO wtf did they make two motor control channels
        debug!("{:?}", dmx_slice);
    }

    fn emit_state(&self, emitter: &mut dyn EmitStateChange) {
        use StateChange::*;
        emitter.emit_rotosphere_q3(Red(self.red));
        emitter.emit_rotosphere_q3(Green(self.green));
        emitter.emit_rotosphere_q3(Blue(self.blue));
        emitter.emit_rotosphere_q3(White(self.white));
        let mut emit_strobe = |ssc| {
            emitter.emit_rotosphere_q3(Strobe(ssc));
        };
        self.strobe.emit_state(&mut emit_strobe);
        emitter.emit_rotosphere_q3(Rotation(self.rotation));
    }

    fn control(
        &mut self,
        msg: ShowControlMessage,
        emitter: &mut dyn EmitStateChange,
    ) -> Option<ShowControlMessage> {
        match msg {
            ShowControlMessage::RotosphereQ3(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    Red(UnipolarFloat),
    Green(UnipolarFloat),
    Blue(UnipolarFloat),
    White(UnipolarFloat),
    Strobe(GenericStrobeStateChange),
    Rotation(BipolarFloat),
}

pub type ControlMessage = StateChange;
