//! A DMX faderboard utility.

use anyhow::Context;
use log::error;
use number::UnipolarFloat;

use super::prelude::*;
use crate::util::unipolar_to_range;

#[derive(Debug)]
pub struct Faderboard {
    channel_count: usize,
    vals: Vec<UnipolarFloat>,
}

impl PatchFixture for Faderboard {
    const NAME: FixtureType = FixtureType("faderboard");
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
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn crate::osc::EmitControlMessage) {
        let (chan, val) = sc;
        if chan >= self.channel_count {
            error!("Channel out of range: {}.", chan);
            return;
        }
        self.vals[chan] = val;
        Self::emit(sc, emitter);
    }
}

impl NonAnimatedFixture for Faderboard {
    fn render(&self, _group_controls: &FixtureGroupControls, dmx_buf: &mut [u8]) {
        for (i, v) in self.vals.iter().enumerate() {
            dmx_buf[i] = unipolar_to_range(0, 255, *v);
        }
    }
}

impl ControllableFixture for Faderboard {
    fn emit_state(&self, emitter: &mut dyn crate::osc::EmitControlMessage) {
        for (i, v) in self.vals.iter().enumerate() {
            Self::emit((i, *v), emitter);
        }
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn crate::osc::EmitControlMessage,
    ) -> anyhow::Result<()> {
        self.handle_state_change(
            *msg.unpack_as::<ControlMessage>().context(Self::NAME)?,
            emitter,
        );
        Ok(())
    }
}

pub type StateChange = (usize, UnipolarFloat);

pub type ControlMessage = StateChange;
