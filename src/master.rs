//! Show-level controls.

use crate::fixture::{
    generic::{GenericStrobe, GenericStrobeStateChange},
    EmitStateChange, FixtureStateChange, Group, StateChange as ShowStateChange,
};

#[derive(Debug, Default)]
pub struct MasterControls {
    strobe: GenericStrobe,
}

impl MasterControls {
    pub fn strobe(&self) -> &GenericStrobe {
        &self.strobe
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
        }
        emitter.emit(ShowStateChange {
            group: Group::none(),
            sc: FixtureStateChange::Master(msg),
        });
    }
}

#[derive(Debug, Clone)]
pub enum StateChange {
    Strobe(GenericStrobeStateChange),
}

pub type ControlMessage = StateChange;
