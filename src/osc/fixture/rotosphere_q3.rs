use super::generic::map_strobe;
use crate::fixture::color::StateChange as ColorStateChange;
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::rotosphere_q3::{RotosphereQ3, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::fixture::color::map_color;
use crate::osc::{ControlMap, HandleStateChange, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "RotosphereQ3";

impl MapControls for RotosphereQ3 {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::RotosphereQ3;
        use StateChange::*;

        map_color(map, GROUP, &wrap_color);
        map_strobe(map, GROUP, "Strobe", &wrap_strobe);
        map.add_bipolar(GROUP, "Rotation", |v| {
            RotosphereQ3(Rotation(bipolar_fader_with_detent(v)))
        });
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> FixtureControlMessage {
    FixtureControlMessage::RotosphereQ3(StateChange::Strobe(sc))
}

fn wrap_color(sc: ColorStateChange) -> FixtureControlMessage {
    FixtureControlMessage::RotosphereQ3(StateChange::Color(sc))
}

impl HandleStateChange<StateChange> for RotosphereQ3 {}
