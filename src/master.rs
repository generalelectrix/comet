//! Show-level controls.

use std::time::Duration;

use number::UnipolarFloat;
use tunnels::clock_server::StaticClockBank;

use crate::osc::HandleStateChange;
use crate::{
    fixture::generic::{GenericStrobe, GenericStrobeStateChange},
    osc::EmitControlMessage,
};

pub use crate::fixture::FixtureGroupControls;

#[derive(Debug, Default)]
pub struct MasterControls {
    strobe: Strobe,
    autopilot: Autopilot,
    pub clock_state: StaticClockBank,
    pub audio_envelope: UnipolarFloat,
}

impl MasterControls {
    pub fn strobe(&self) -> &Strobe {
        &self.strobe
    }

    pub fn autopilot(&self) -> &Autopilot {
        &self.autopilot
    }

    pub fn update(&mut self, delta_t: Duration) {
        self.autopilot.update(delta_t);
    }

    pub fn emit_state(&self, emitter: &dyn EmitControlMessage) {
        use StateChange::*;
        let mut emit_strobe = |ssc| {
            Self::emit(Strobe(ssc), emitter);
        };
        self.strobe.state.emit_state(&mut emit_strobe);
    }

    pub fn control(&mut self, msg: ControlMessage, emitter: &dyn EmitControlMessage) {
        use StateChange::*;
        match msg {
            Strobe(sc) => self.strobe.state.handle_state_change(sc),
            UseMasterStrobeRate(v) => self.strobe.use_master_rate = v,
            AutopilotOn(v) => self.autopilot.on = v,
            AutopilotSoundActive(v) => self.autopilot.sound_active = v,
        }
        Self::emit(msg, emitter);
    }
}

#[derive(Debug)]
pub struct Autopilot {
    on: bool,
    sound_active: bool,
    program_count: usize,
    program: usize,
    program_change_interval: Duration,
    program_age: Duration,
}

impl Default for Autopilot {
    fn default() -> Self {
        Self {
            on: false,
            sound_active: false,
            program_count: 32,
            program: 0,
            program_change_interval: Duration::from_secs(60),
            program_age: Duration::ZERO,
        }
    }
}

impl Autopilot {
    fn update(&mut self, delta_t: Duration) {
        self.program_age += delta_t;
        if self.program_age >= self.program_change_interval {
            self.program = (self.program + 1) % self.program_count;
            self.program_age = Duration::ZERO;
        }
    }

    pub fn on(&self) -> bool {
        self.on
    }

    pub fn program(&self) -> usize {
        self.program
    }

    pub fn sound_active(&self) -> bool {
        self.sound_active
    }
}

#[derive(Debug, Clone)]
pub enum StateChange {
    Strobe(GenericStrobeStateChange),
    UseMasterStrobeRate(bool),
    AutopilotOn(bool),
    AutopilotSoundActive(bool),
}

pub type ControlMessage = StateChange;

#[derive(Debug, Default)]
pub struct Strobe {
    pub state: GenericStrobe,
    pub use_master_rate: bool,
}
