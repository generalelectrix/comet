//! Control profile for a Radiance hazer.
//! Probably fine for any generic 2-channel hazer.
use anyhow::{Context, Result};
use std::{collections::HashMap, time::Duration};

use number::UnipolarFloat;

use super::{
    generic::Timer, ControlMessagePayload, ControllableFixture, EmitFixtureStateChange,
    FixtureControlMessage, NonAnimatedFixture, PatchFixture,
};
use crate::{master::FixtureGroupControls, util::unipolar_to_range};

#[derive(Default, Debug)]
pub struct Radiance {
    haze: UnipolarFloat,
    fan: UnipolarFloat,
    timer: Option<Timer>,
}

impl PatchFixture for Radiance {
    const NAME: &'static str = "radiance";
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
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        match sc {
            Haze(v) => self.haze = v,
            Fan(v) => self.fan = v,
        };
        emitter.emit_radiance(sc);
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
    fn update(&mut self, delta_t: Duration) {
        if let Some(timer) = self.timer.as_mut() {
            timer.update(delta_t);
        }
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
    ) -> anyhow::Result<()> {
        self.handle_state_change(
            *msg.unpack_as::<ControlMessage>().context(Self::NAME)?,
            emitter,
        );
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
