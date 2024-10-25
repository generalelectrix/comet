//! A DMX faderboard utility.

use log::error;

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Debug)]
pub struct Faderboard {
    controls: GroupControlMap<ControlMessage>,
    channel_count: usize,
    vals: Vec<UnipolarFloat>,
}

impl PatchFixture for Faderboard {
    const NAME: FixtureType = FixtureType("Faderboard");
    fn channel_count(&self) -> usize {
        self.channel_count
    }
}

const DEFAULT_CHANNEL_COUNT: usize = 16;

impl Default for Faderboard {
    fn default() -> Self {
        let mut controls = GroupControlMap::default();
        CONTROLS.map(&mut controls, |index, val| Ok((index, val)));
        Self {
            controls,
            vals: vec![UnipolarFloat::ZERO; DEFAULT_CHANNEL_COUNT],
            channel_count: DEFAULT_CHANNEL_COUNT,
        }
    }
}

impl Faderboard {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &FixtureStateEmitter) {
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
    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        for (i, v) in self.vals.iter().enumerate() {
            Self::emit((i, *v), emitter);
        }
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        let Some((ctl, _)) = self.controls.handle(msg)? else {
            return Ok(true);
        };
        self.handle_state_change(ctl, emitter);
        Ok(true)
    }
}

pub type StateChange = (usize, UnipolarFloat);

pub type ControlMessage = StateChange;

const CONTROLS: FaderArray = FaderArray { control: "Fader" };

impl HandleOscStateChange<StateChange> for Faderboard {}
