use crate::fixture::hypnotic::{Hypnotic, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{ControlMap, HandleStateChange, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "Hypnotic";

const RED_LASER_ON: Button = button(GROUP, "RedLaserOn");
const GREEN_LASER_ON: Button = button(GROUP, "GreenLaserOn");
const BLUE_LASER_ON: Button = button(GROUP, "BlueLaserOn");

impl MapControls for Hypnotic {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::Hypnotic;
        use StateChange::*;
        RED_LASER_ON.map_state(map, |v| Hypnotic(RedLaserOn(v)));
        GREEN_LASER_ON.map_state(map, |v| Hypnotic(GreenLaserOn(v)));
        BLUE_LASER_ON.map_state(map, |v| Hypnotic(BlueLaserOn(v)));

        map.add_bipolar(GROUP, "Rotation", |v| {
            Hypnotic(Rotation(bipolar_fader_with_detent(v)))
        });
    }
}

impl HandleStateChange<StateChange> for Hypnotic {}
