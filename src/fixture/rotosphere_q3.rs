//! Control profle for the Chauvet Rotosphere Q3, aka Son Of Spherion.

use number::{BipolarFloat, UnipolarFloat};

use super::generic::{GenericStrobe, GenericStrobeStateChange};
use super::{EmitFixtureStateChange, Fixture, FixtureControlMessage, PatchFixture};
use crate::master::{Autopilot, MasterControls};
use crate::util::{bipolar_to_split_range, unipolar_to_range};

#[derive(Default, Debug)]
pub struct RotosphereQ3 {
    red: UnipolarFloat,
    green: UnipolarFloat,
    blue: UnipolarFloat,
    white: UnipolarFloat,
    strobe: GenericStrobe,
    rotation: BipolarFloat,
}

impl PatchFixture for RotosphereQ3 {
    fn channel_count(&self) -> usize {
        9
    }
}

impl RotosphereQ3 {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
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

    fn render_autopilot(&self, autopilot: &Autopilot, dmx_buf: &mut [u8]) {
        dmx_buf[0] = 0;
        dmx_buf[1] = 0;
        dmx_buf[2] = 0;
        dmx_buf[3] = 0;
        dmx_buf[4] = 0;
        dmx_buf[5] = 0;
        dmx_buf[6] = match autopilot.program() % 5 {
            0 => 212,
            1 => 221,
            2 => 230,
            3 => 239,
            4 => 248,
            _ => 212,
        };
        dmx_buf[7] = if autopilot.sound_active() {
            255
        } else {
            50 // TODO is this a good value?  Too slow?
        };
        dmx_buf[8] = 30; // TODO is this a good value?  Too slow?
    }
}

impl Fixture for RotosphereQ3 {
    fn render(&self, master: &MasterControls, dmx_buf: &mut [u8]) {
        if master.autopilot().on() {
            self.render_autopilot(master.autopilot(), dmx_buf);
            return;
        }
        dmx_buf[0] = unipolar_to_range(0, 255, self.red);
        dmx_buf[1] = unipolar_to_range(0, 255, self.green);
        dmx_buf[2] = unipolar_to_range(0, 255, self.blue);
        dmx_buf[3] = unipolar_to_range(0, 255, self.white);
        dmx_buf[4] = self
            .strobe
            .render_range_with_master(master.strobe(), 0, 1, 250);
        dmx_buf[5] = bipolar_to_split_range(self.rotation, 1, 127, 129, 255, 0);
        dmx_buf[6] = 0;
        dmx_buf[7] = 0;
        dmx_buf[8] = 0;
    }

    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
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
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::RotosphereQ3(msg) => {
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
