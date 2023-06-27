use crate::fixture::hypnotic::{Hypnotic, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::{ControlMap, HandleStateChange, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "Hypnotic";

impl MapControls for Hypnotic {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::Hypnotic;
        use StateChange::*;
        map.add_bool(GROUP, "RedLaserOn", |v| Hypnotic(RedLaserOn(v)));
        map.add_bool(GROUP, "GreenLaserOn", |v| Hypnotic(GreenLaserOn(v)));
        map.add_bool(GROUP, "BlueLaserOn", |v| Hypnotic(BlueLaserOn(v)));
        map.add_bipolar(GROUP, "Rotation", |v| {
            Hypnotic(Rotation(bipolar_fader_with_detent(v)))
        });
    }
}

impl HandleStateChange<StateChange> for Hypnotic {}
