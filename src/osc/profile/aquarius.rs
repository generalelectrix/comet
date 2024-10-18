use crate::fixture::aquarius::{Aquarius, StateChange};
use crate::fixture::{prelude::*, ControlMessagePayload};
use crate::osc::basic_controls::{button, Button};
use crate::osc::{ControlMap, HandleOscStateChange, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "Aquarius";

const LAMP_ON: Button = button(GROUP, "LampOn");

impl MapControls for Aquarius {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        use StateChange::*;
        LAMP_ON.map_state(map, |v| ControlMessagePayload::fixture(LampOn(v)));
        map.add_bipolar(GROUP, "Rotation", |v| {
            ControlMessagePayload::fixture(Rotation(bipolar_fader_with_detent(v)))
        });
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        vec![(GROUP.to_string(), Self::NAME)]
    }
}

impl HandleOscStateChange<StateChange> for Aquarius {}
