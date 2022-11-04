//! Control profile for a Radiance hazer.
//! Probably fine for any generic 2-channel hazer.

use number::UnipolarFloat;

use super::{EmitFixtureStateChange, Fixture, FixtureControlMessage, PatchFixture};
use crate::util::unipolar_to_range;

#[derive(Default, Debug)]
pub struct Radiance {
    haze: UnipolarFloat,
    fan: UnipolarFloat,
}

impl PatchFixture for Radiance {
    const CHANNEL_COUNT: usize = 2;
}

impl Radiance {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        match sc {
            Haze(v) => self.haze = v,
            Fan(v) => self.fan = v,
        };
        emitter.emit_radiance(sc);
    }
}

impl Fixture for Radiance {
    fn render(&self, dmx_buf: &mut [u8]) {
        dmx_buf[0] = unipolar_to_range(0, 255, self.haze);
        dmx_buf[1] = unipolar_to_range(0, 255, self.fan);
    }

    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        emitter.emit_radiance(Haze(self.haze));
        emitter.emit_radiance(Fan(self.fan));
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::Radiance(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    Haze(UnipolarFloat),
    Fan(UnipolarFloat),
}

// Venus has no controls that are not represented as state changes.
pub type ControlMessage = StateChange;
