use std::time::Duration;

use crate::aquarius::{
    ControlMessage as AquariusControlMessage, StateChange as AquariusStateChange,
};
use crate::comet::{ControlMessage as CometControlMessage, StateChange as CometStateChange};
use crate::h2o::{ControlMessage as H2OControlMessage, StateChange as H2OStateChange};
use crate::lumasphere::{
    ControlMessage as LumasphereControlMessage, StateChange as LumasphereStateChange,
};
use crate::venus::{ControlMessage as VenusControlMessage, StateChange as VenusStateChange};

pub trait EmitStateChange {
    fn emit(&mut self, sc: StateChange);

    fn emit_comet(&mut self, sc: CometStateChange) {
        self.emit(StateChange::Comet(sc));
    }

    fn emit_lumasphere(&mut self, sc: LumasphereStateChange) {
        self.emit(StateChange::Lumasphere(sc));
    }

    fn emit_venus(&mut self, sc: VenusStateChange) {
        self.emit(StateChange::Venus(sc));
    }

    fn emit_h2o(&mut self, sc: H2OStateChange) {
        self.emit(StateChange::H2O(sc));
    }

    fn emit_aquarius(&mut self, sc: AquariusStateChange) {
        self.emit(StateChange::Aquarius(sc));
    }
}

#[derive(Clone, Debug)]
pub enum StateChange {
    Comet(CometStateChange),
    Lumasphere(LumasphereStateChange),
    Venus(VenusStateChange),
    H2O(H2OStateChange),
    Aquarius(AquariusStateChange),
}

#[derive(Clone, Debug)]
pub enum ControlMessage {
    Comet(CometControlMessage),
    Lumasphere(LumasphereControlMessage),
    Venus(VenusControlMessage),
    H2O(H2OControlMessage),
    Aquarius(AquariusControlMessage),
}

pub trait Fixture {
    fn emit_state(&self, emitter: &mut dyn EmitStateChange);

    /// Potentially process the provided control message.
    /// If this fixture will not process it, return it back to the caller.
    fn control(
        &mut self,
        msg: ControlMessage,
        emitter: &mut dyn EmitStateChange,
    ) -> Option<ControlMessage>;

    fn update(&mut self, _: Duration) {}

    /// Render into the provided DMX universe.
    fn render(&self, dmx_buffer: &mut [u8]);
}
