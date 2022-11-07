//! Show-level controls.

use std::time::Duration;

use crate::fixture::{
    generic::{GenericStrobe, GenericStrobeStateChange},
    EmitStateChange, FixtureStateChange, Group, StateChange as ShowStateChange,
};

#[derive(Debug, Default)]
pub struct MasterControls {
    strobe: GenericStrobe,
    autopilot: Autopilot,
}

impl MasterControls {
    pub fn strobe(&self) -> &GenericStrobe {
        &self.strobe
    }

    pub fn autopilot(&self) -> &Autopilot {
        &self.autopilot
    }

    pub fn update(&mut self, delta_t: Duration) {
        self.autopilot.update(delta_t);
    }

    pub fn emit_state(&self, emitter: &mut dyn EmitStateChange) {
        use StateChange::*;
        let mut emit_strobe = |ssc| {
            emitter.emit(ShowStateChange {
                group: Group::none(),
                sc: FixtureStateChange::Master(Strobe(ssc)),
            });
        };
        self.strobe.emit_state(&mut emit_strobe);
    }

    pub fn control(&mut self, msg: ControlMessage, emitter: &mut dyn EmitStateChange) {
        use StateChange::*;
        match msg {
            Strobe(sc) => self.strobe.handle_state_change(sc),
            AutopilotOn(v) => self.autopilot.on = v,
            AutopilotSoundActive(v) => self.autopilot.sound_active = v,
        }
        emitter.emit(ShowStateChange {
            group: Group::none(),
            sc: FixtureStateChange::Master(msg),
        });
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
    AutopilotOn(bool),
    AutopilotSoundActive(bool),
}

pub type ControlMessage = StateChange;
