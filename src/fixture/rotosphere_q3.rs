//! Control profle for the Chauvet Rotosphere Q3, aka Son Of Spherion.

use number::BipolarFloat;

use super::color::{Color, Model as ColorModel, StateChange as ColorStateChange};
use super::generic::{GenericStrobe, GenericStrobeStateChange};
use super::{
    ControllableFixture, EmitFixtureStateChange, FixtureControlMessage, NonAnimatedFixture,
    PatchFixture,
};
use crate::master::{Autopilot, MasterControls};
use crate::util::bipolar_to_split_range;

#[derive(Debug)]
pub struct RotosphereQ3 {
    color: Color,
    strobe: GenericStrobe,
    rotation: BipolarFloat,
}

impl Default for RotosphereQ3 {
    fn default() -> Self {
        Self {
            color: Color::from_model(ColorModel::rgbw()),
            strobe: GenericStrobe::default(),
            rotation: BipolarFloat::default(),
        }
    }
}

impl PatchFixture for RotosphereQ3 {
    const NAME: &'static str = "rotosphere_q3";
    fn channel_count(&self) -> usize {
        9
    }
}

impl RotosphereQ3 {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        match sc {
            Color(c) => self.color.update_state(c),
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

impl NonAnimatedFixture for RotosphereQ3 {
    fn render(&self, master: &MasterControls, dmx_buf: &mut [u8]) {
        if master.autopilot().on() {
            self.render_autopilot(master.autopilot(), dmx_buf);
            return;
        }
        self.color.render(master, &mut dmx_buf[0..4]);
        dmx_buf[4] = self
            .strobe
            .render_range_with_master(master.strobe(), 0, 1, 250);
        dmx_buf[5] = bipolar_to_split_range(self.rotation, 1, 127, 129, 255, 0);
        dmx_buf[6] = 0;
        dmx_buf[7] = 0;
        dmx_buf[8] = 0;
    }
}

impl ControllableFixture for RotosphereQ3 {
    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        let mut emit_color = |sc| {
            emitter.emit_rotosphere_q3(Color(sc));
        };
        self.color.state(&mut emit_color);
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
    Color(ColorStateChange),
    Strobe(GenericStrobeStateChange),
    Rotation(BipolarFloat),
}

pub type ControlMessage = StateChange;
