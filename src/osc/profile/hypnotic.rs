use crate::fixture::hypnotic::{Hypnotic, StateChange};

use crate::fixture::PatchAnimatedFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{GroupControlMap, HandleOscStateChange};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = Hypnotic::NAME.0;

const RED_LASER_ON: Button = button(GROUP, "RedLaserOn");
const GREEN_LASER_ON: Button = button(GROUP, "GreenLaserOn");
const BLUE_LASER_ON: Button = button(GROUP, "BlueLaserOn");

impl Hypnotic {
    fn group(&self) -> &'static str {
        GROUP
    }
    fn map_controls(&self, map: &mut GroupControlMap<ControlMessagePayload>) {
        use StateChange::*;
        RED_LASER_ON.map_state(map, |v| ControlMessagePayload::fixture(RedLaserOn(v)));
        GREEN_LASER_ON.map_state(map, |v| ControlMessagePayload::fixture(GreenLaserOn(v)));
        BLUE_LASER_ON.map_state(map, |v| ControlMessagePayload::fixture(BlueLaserOn(v)));

        map.add_bipolar("Rotation", |v| {
            ControlMessagePayload::fixture(Rotation(bipolar_fader_with_detent(v)))
        });
    }
}

impl HandleOscStateChange<StateChange> for Hypnotic {}
