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
    pub clock_state: StaticClockBank,
    pub audio_envelope: UnipolarFloat,
}

impl MasterControls {
    pub fn strobe(&self) -> &Strobe {
        &self.strobe
    }

    pub fn update(&mut self, _delta_t: Duration) {}

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
        }
        Self::emit(msg, emitter);
    }
}

#[derive(Debug, Clone)]
pub enum StateChange {
    Strobe(GenericStrobeStateChange),
    UseMasterStrobeRate(bool),
}

pub type ControlMessage = StateChange;

#[derive(Debug, Default)]
pub struct Strobe {
    pub state: GenericStrobe,
    pub use_master_rate: bool,
}
