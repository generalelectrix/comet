use super::generic::map_strobe;
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::rotosphere_q3::{RotosphereQ3, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::{ControlMap, HandleStateChange, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "RotosphereQ3";

impl MapControls for RotosphereQ3 {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::RotosphereQ3;
        use StateChange::*;

        map.add_unipolar(GROUP, "Red", |v| RotosphereQ3(Red(v)));
        map.add_unipolar(GROUP, "Green", |v| RotosphereQ3(Green(v)));
        map.add_unipolar(GROUP, "Blue", |v| RotosphereQ3(Blue(v)));
        map.add_unipolar(GROUP, "White", |v| RotosphereQ3(White(v)));
        map_strobe(map, GROUP, "Strobe", &wrap_strobe);
        map.add_bipolar(GROUP, "Rotation", |v| {
            RotosphereQ3(Rotation(bipolar_fader_with_detent(v)))
        });
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> FixtureControlMessage {
    FixtureControlMessage::RotosphereQ3(StateChange::Strobe(sc))
}

impl HandleStateChange<StateChange> for RotosphereQ3 {}
