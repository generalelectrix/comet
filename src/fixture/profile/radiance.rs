//! Control profile for a Radiance hazer.
//! Probably fine for any generic 2-channel hazer.
use anyhow::Result;
use std::{collections::HashMap, time::Duration};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Default, Debug)]
pub struct Radiance {
    controls: GroupControlMap<ControlMessage>,
    haze: UnipolarFloat,
    fan: UnipolarFloat,
    timer: Option<Timer>,
}

impl PatchFixture for Radiance {
    const NAME: FixtureType = FixtureType("Radiance");
    fn channel_count(&self) -> usize {
        2
    }

    fn new(options: &HashMap<String, String>) -> Result<Self> {
        let mut s = Self::default();
        if options.contains_key("use_timer") {
            s.timer = Some(Timer::from_options(options)?);
        }
        Ok(s)
    }
}

impl Radiance {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        match sc {
            Haze(v) => self.haze = v,
            Fan(v) => self.fan = v,
        };
        Self::emit(sc, emitter);
    }
}

impl NonAnimatedFixture for Radiance {
    fn render(&self, _group_controls: &FixtureGroupControls, dmx_buf: &mut [u8]) {
        if let Some(timer) = self.timer.as_ref() {
            if !timer.is_on() {
                dmx_buf[0] = 0;
                dmx_buf[1] = 0;
                return;
            }
        }
        dmx_buf[0] = unipolar_to_range(0, 255, self.haze);
        dmx_buf[1] = unipolar_to_range(0, 255, self.fan);
    }
}

impl ControllableFixture for Radiance {
    fn populate_controls(&mut self) {
        Self::map_controls(&mut self.controls);
    }

    fn update(&mut self, delta_t: Duration) {
        if let Some(timer) = self.timer.as_mut() {
            timer.update(delta_t);
        }
    }

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        Self::emit(Haze(self.haze), emitter);
        Self::emit(Fan(self.fan), emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        let Some((ctl, _)) = self.controls.handle(msg)? else {
            return Ok(());
        };
        self.handle_state_change(ctl, emitter);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    Haze(UnipolarFloat),
    Fan(UnipolarFloat),
}

// Venus has no controls that are not represented as state changes.
pub type ControlMessage = StateChange;

impl Radiance {
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;
        map.add_unipolar("Haze", Haze);
        map.add_unipolar("Fan", Fan);
    }
}

impl HandleOscStateChange<StateChange> for Radiance {}
