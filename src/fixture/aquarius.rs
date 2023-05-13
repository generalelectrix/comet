//! Intuitive control profile for the American DJ Aquarius 250.

use number::BipolarFloat;

use super::{
    ControllableFixture, EmitFixtureStateChange, Fixture, FixtureControlMessage,
    NonAnimatedFixture, PatchFixture,
};
use crate::{master::MasterControls, util::bipolar_to_split_range};

#[derive(Default, Debug)]
pub struct Aquarius {
    lamp_on: bool,
    rotation: BipolarFloat,
}

impl PatchFixture for Aquarius {
    const NAME: &'static str = "aquarius";
    fn channel_count(&self) -> usize {
        2
    }
}

impl Aquarius {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        match sc {
            LampOn(v) => self.lamp_on = v,
            Rotation(v) => self.rotation = v,
        };
        emitter.emit_aquarius(sc);
    }
}

impl NonAnimatedFixture for Aquarius {
    fn render(&self, _master_controls: &MasterControls, dmx_buf: &mut [u8]) {
        dmx_buf[0] = bipolar_to_split_range(self.rotation, 130, 8, 132, 255, 0);
        dmx_buf[1] = if self.lamp_on { 255 } else { 0 };
    }
}

impl ControllableFixture for Aquarius {
    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        emitter.emit_aquarius(LampOn(self.lamp_on));
        emitter.emit_aquarius(Rotation(self.rotation));
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::Aquarius(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    LampOn(bool),
    Rotation(BipolarFloat),
}

// Aquarius has no controls that are not represented as state changes.
pub type ControlMessage = StateChange;
