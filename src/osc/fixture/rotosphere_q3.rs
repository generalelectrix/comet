use super::generic::map_strobe;
use crate::fixture::color::StateChange as ColorStateChange;
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::rotosphere_q3::{RotosphereQ3, StateChange};
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchAnimatedFixture;
use crate::osc::fixture::color::map_color;
use crate::osc::{ControlMap, HandleStateChange, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "RotosphereQ3";

impl MapControls for RotosphereQ3 {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        use StateChange::*;

        map_color(map, GROUP, &wrap_color);
        map_strobe(map, GROUP, "Strobe", &wrap_strobe);
        map.add_bipolar(GROUP, "Rotation", |v| {
            ControlMessagePayload::fixture(Rotation(bipolar_fader_with_detent(v)))
        });
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        vec![(GROUP.to_string(), Self::NAME)]
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessagePayload {
    ControlMessagePayload::fixture(StateChange::Strobe(sc))
}

fn wrap_color(sc: ColorStateChange) -> ControlMessagePayload {
    ControlMessagePayload::fixture(StateChange::Color(sc))
}

impl HandleStateChange<StateChange> for RotosphereQ3 {}
