use crate::fixture::hypnotic::{Hypnotic, StateChange};
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchAnimatedFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{ControlMap, HandleOscStateChange, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "Hypnotic";

const RED_LASER_ON: Button = button(GROUP, "RedLaserOn");
const GREEN_LASER_ON: Button = button(GROUP, "GreenLaserOn");
const BLUE_LASER_ON: Button = button(GROUP, "BlueLaserOn");

impl MapControls for Hypnotic {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        use StateChange::*;
        RED_LASER_ON.map_state(map, |v| ControlMessagePayload::fixture(RedLaserOn(v)));
        GREEN_LASER_ON.map_state(map, |v| ControlMessagePayload::fixture(GreenLaserOn(v)));
        BLUE_LASER_ON.map_state(map, |v| ControlMessagePayload::fixture(BlueLaserOn(v)));

        map.add_bipolar(GROUP, "Rotation", |v| {
            ControlMessagePayload::fixture(Rotation(bipolar_fader_with_detent(v)))
        });
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        vec![(GROUP.to_string(), Self::NAME)]
    }
}

impl HandleOscStateChange<StateChange> for Hypnotic {}
