use crate::fixture::faderboard::{Faderboard, StateChange};
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchFixture;
use crate::osc::fader_array::FaderArray;
use crate::osc::{ControlMap, HandleOscStateChange, MapControls};

const GROUP: &str = "Faderboard";

const CONTROLS: FaderArray = FaderArray {
    group: GROUP,
    control: "Fader",
};

impl MapControls for Faderboard {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        CONTROLS.map(map, |index, val| {
            Ok(ControlMessagePayload::fixture((index, val)))
        })
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        vec![(GROUP.to_string(), Self::NAME)]
    }
}

impl HandleOscStateChange<StateChange> for Faderboard {}
