//! A DMX faderboard utility.

use log::error;
use number::UnipolarFloat;

use super::{EmitFixtureStateChange, Fixture, FixtureControlMessage, PatchFixture};
use crate::{master::MasterControls, util::unipolar_to_range};

#[derive(Debug)]
pub struct Faderboard {
    channel_count: usize,
    vals: Vec<UnipolarFloat>,
}

impl PatchFixture for Faderboard {
    fn channel_count(&self) -> usize {
        self.channel_count
    }
}

const DEFAULT_CHANNEL_COUNT: usize = 16;

impl Default for Faderboard {
    fn default() -> Self {
        Self {
            vals: vec![UnipolarFloat::ZERO; DEFAULT_CHANNEL_COUNT],
            channel_count: DEFAULT_CHANNEL_COUNT,
        }
    }
}

impl Faderboard {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
        let (chan, val) = sc;
        if chan >= self.channel_count {
            error!("Channel out of range: {}.", chan);
            return;
        }
        self.vals[chan] = val;
        emitter.emit_faderboard(sc);
    }
}

impl Fixture for Faderboard {
    fn render(&self, _master_controls: &MasterControls, dmx_buf: &mut [u8]) {
        for (i, v) in self.vals.iter().enumerate() {
            dmx_buf[i] = unipolar_to_range(0, 255, *v);
        }
    }

    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        for (i, v) in self.vals.iter().enumerate() {
            emitter.emit_faderboard((i, *v));
        }
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::Faderboard(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

pub type StateChange = (usize, UnipolarFloat);

pub type ControlMessage = StateChange;
