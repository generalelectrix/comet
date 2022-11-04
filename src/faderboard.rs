//! A DMX faderboard utility.

use log::{debug, error};
use number::UnipolarFloat;

use crate::dmx::DmxAddr;
use crate::fixture::{ControlMessage as ShowControlMessage, EmitStateChange, Fixture};
use crate::util::unipolar_to_range;

pub struct Faderboard {
    channels: usize,
    start_index: usize,
    vals: Vec<UnipolarFloat>,
}

impl Faderboard {
    pub fn new(channels: usize, start_addr: DmxAddr) -> Self {
        Self {
            channels,
            start_index: start_addr - 1,
            vals: vec![UnipolarFloat::ZERO; channels],
        }
    }

    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitStateChange) {
        let (chan, val) = sc;
        if chan >= self.channels {
            error!("Channel out of range: {}.", chan);
            return;
        }
        self.vals[chan] = val;
        emitter.emit_faderboard(sc);
    }
}

impl Fixture for Faderboard {
    fn render(&self, dmx_univ: &mut [u8]) {
        let dmx_slice = &mut dmx_univ[self.start_index..self.start_index + self.channels];
        for (i, v) in self.vals.iter().enumerate() {
            dmx_slice[i] = unipolar_to_range(0, 255, *v);
        }
        debug!("{:?}", dmx_slice);
    }

    fn emit_state(&self, emitter: &mut dyn EmitStateChange) {
        for (i, v) in self.vals.iter().enumerate() {
            emitter.emit_faderboard((i, *v));
        }
    }

    fn control(
        &mut self,
        msg: ShowControlMessage,
        emitter: &mut dyn EmitStateChange,
    ) -> Option<ShowControlMessage> {
        match msg {
            ShowControlMessage::Faderboard(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

pub type StateChange = (usize, UnipolarFloat);

pub type ControlMessage = StateChange;
