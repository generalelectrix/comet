use crate::fixture::freedom_fries::{FreedomFries as FreedomFriesFixture, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::{ControlMap, HandleStateChange, MapControls};
use crate::util::unipolar_to_range;

const GROUP: &str = "FreedomFries";

impl MapControls for FreedomFriesFixture {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::FreedomFries;
        use StateChange::*;

        map.add_unipolar(GROUP, "Dimmer", |v| FreedomFries(Dimmer(v)));
        map.add_unipolar(GROUP, "Speed", |v| FreedomFries(Speed(v)));
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

impl HandleStateChange<StateChange> for FreedomFriesFixture {}
