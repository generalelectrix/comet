//! Control profile for a Radiance hazer.
//! Probably fine for any generic 2-channel hazer.
use anyhow::Result;
use std::{collections::HashMap, time::Duration};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Debug)]
pub struct Radiance {
    haze: UnipolarChannel,
    fan: UnipolarChannel,
    timer: Option<Timer>,
}

impl Default for Radiance {
    fn default() -> Self {
        Self {
            haze: Unipolar::full_channel("Haze", 0),
            fan: Unipolar::full_channel("Fan", 1),
            timer: None,
        }
    }
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

impl NonAnimatedFixture for Radiance {
    fn render(&self, _group_controls: &FixtureGroupControls, dmx_buf: &mut [u8]) {
        if let Some(timer) = self.timer.as_ref() {
            if !timer.is_on() {
                dmx_buf[0] = 0;
                dmx_buf[1] = 0;
                return;
            }
        }
        self.haze.render_no_anim(dmx_buf);
        self.fan.render_no_anim(dmx_buf);
    }
}

impl ControllableFixture for Radiance {
    fn populate_controls(&mut self) {}

    fn update(&mut self, delta_t: Duration) {
        if let Some(timer) = self.timer.as_mut() {
            timer.update(delta_t);
        }
    }

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.haze.emit_state(emitter);
        self.fan.emit_state(emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        if self.haze.control_ok(msg, emitter)? {
            return Ok(true);
        }
        if self.fan.control_ok(msg, emitter)? {
            return Ok(true);
        }
        Ok(false)
    }
}
