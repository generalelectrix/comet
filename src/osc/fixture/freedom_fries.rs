use crate::fixture::color::StateChange as ColorStateChange;
use crate::fixture::freedom_fries::{FreedomFries as FreedomFriesFixture, StateChange};
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::FixtureControlMessage;
use crate::osc::fixture::color::map_color;
use crate::osc::fixture::generic::map_strobe;
use crate::osc::{ControlMap, HandleStateChange, MapControls};
use crate::util::unipolar_to_range;

const GROUP: &str = "FreedomFries";

impl MapControls for FreedomFriesFixture {
    fn group(&self) -> &'static str {
        GROUP
    }
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::FreedomFries;
        use StateChange::*;

        map.add_unipolar(GROUP, "Dimmer", |v| FreedomFries(Dimmer(v)));
        map_color(map, GROUP, &wrap_color);
        map_strobe(map, GROUP, "Strobe", &wrap_strobe);
        map.add_unipolar(GROUP, "Speed", |v| FreedomFries(Speed(v)));
        map.add_bool(GROUP, "RunProgram", |v| FreedomFries(RunProgram(v)));
        map.add_unipolar(GROUP, "Program", |v| {
            FreedomFries(Program(unipolar_to_range(
                0,
                FreedomFriesFixture::PROGRAM_COUNT as u8 - 1,
                v,
            ) as usize))
        });
        map.add_bool(GROUP, "ProgramCycleAll", |v| {
            FreedomFries(ProgramCycleAll(v))
        });
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> FixtureControlMessage {
    FixtureControlMessage::FreedomFries(StateChange::Strobe(sc))
}

fn wrap_color(sc: ColorStateChange) -> FixtureControlMessage {
    FixtureControlMessage::FreedomFries(StateChange::Color(sc))
}

impl HandleStateChange<StateChange> for FreedomFriesFixture {}
