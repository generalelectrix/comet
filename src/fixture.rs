use std::time::Duration;

use crate::aquarius::{
    ControlMessage as AquariusControlMessage, StateChange as AquariusStateChange,
};
use crate::comet::{ControlMessage as CometControlMessage, StateChange as CometStateChange};
use crate::faderboard::{
    ControlMessage as FaderboardControlMessage, StateChange as FaderboardStateChange,
};
use crate::freedom_fries::{
    ControlMessage as FreedomFriesControlMessage, StateChange as FreedomFriesStateChange,
};
use crate::h2o::{ControlMessage as H2OControlMessage, StateChange as H2OStateChange};
use crate::lumasphere::{
    ControlMessage as LumasphereControlMessage, StateChange as LumasphereStateChange,
};
use crate::radiance::{
    ControlMessage as RadianceControlMessage, StateChange as RadianceStateChange,
};
use crate::rotosphere_q3::{
    ControlMessage as RotosphereQ3ControlMessage, StateChange as RotosphereQ3StateChange,
};
use crate::swarmolon::{
    ControlMessage as SwarmolonControlMessage, StateChange as SwarmolonStateChange,
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

    fn emit_radiance(&mut self, sc: RadianceStateChange) {
        self.emit(StateChange::Radiance(sc));
    }

    fn emit_swarmolon(&mut self, sc: SwarmolonStateChange) {
        self.emit(StateChange::Swarmolon(sc));
    }

    fn emit_rotosphere_q3(&mut self, sc: RotosphereQ3StateChange) {
        self.emit(StateChange::RotosphereQ3(sc));
    }

    fn emit_freedom_fries(&mut self, sc: FreedomFriesStateChange) {
        self.emit(StateChange::FreedomFries(sc));
    }

    fn emit_faderboard(&mut self, sc: FaderboardStateChange) {
        self.emit(StateChange::Faderboard(sc));
    }
}

#[derive(Clone, Debug)]
pub enum StateChange {
    Comet(CometStateChange),
    Lumasphere(LumasphereStateChange),
    Venus(VenusStateChange),
    H2O(H2OStateChange),
    Aquarius(AquariusStateChange),
    Radiance(RadianceStateChange),
    Swarmolon(SwarmolonStateChange),
    RotosphereQ3(RotosphereQ3StateChange),
    FreedomFries(FreedomFriesStateChange),
    Faderboard(FaderboardStateChange),
}

#[derive(Clone, Debug)]
pub enum ControlMessage {
    Comet(CometControlMessage),
    Lumasphere(LumasphereControlMessage),
    Venus(VenusControlMessage),
    H2O(H2OControlMessage),
    Aquarius(AquariusControlMessage),
    Radiance(RadianceControlMessage),
    Swarmolon(SwarmolonControlMessage),
    RotosphereQ3(RotosphereQ3ControlMessage),
    FreedomFries(FreedomFriesControlMessage),
    Faderboard(FaderboardControlMessage),
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
