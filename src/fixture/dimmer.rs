//! Control profile for a dimmer.

use number::UnipolarFloat;

use super::{EmitFixtureStateChange, Fixture, FixtureControlMessage, PatchFixture};
use crate::{master::MasterControls, util::unipolar_to_range};

#[derive(Default, Debug)]
pub struct Dimmer(UnipolarFloat);

impl PatchFixture for Dimmer {
    const NAME: &'static str = "dimmer";
    fn channel_count(&self) -> usize {
        1
    }
}

impl Dimmer {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
        self.0 = sc;
        emitter.emit_dimmer(sc);
    }
}

impl Fixture for Dimmer {
    fn render(&self, _master_controls: &MasterControls, dmx_buf: &mut [u8]) {
        dmx_buf[0] = unipolar_to_range(0, 255, self.0);
    }

    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        emitter.emit_dimmer(self.0);
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::Dimmer(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

pub type StateChange = UnipolarFloat;

// Venus has no controls that are not represented as state changes.
pub type ControlMessage = StateChange;
