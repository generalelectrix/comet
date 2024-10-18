use super::generic::map_strobe;
use crate::fixture::color::StateChange as ColorStateChange;
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::rotosphere_q3::{RotosphereQ3, StateChange};

use crate::fixture::PatchAnimatedFixture;
use crate::osc::profile::color::map_color;
use crate::osc::{GroupControlMap, HandleOscStateChange};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "RotosphereQ3";

impl RotosphereQ3 {
    fn group(&self) -> &'static str {
        GROUP
    }
    fn map_controls(&self, map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;

        map_color(map, &wrap_color);
        map_strobe(map, "Strobe", &wrap_strobe);
        map.add_bipolar("Rotation", |v| {
            Rotation(bipolar_fader_with_detent(v))
        });
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessagePayload {
    StateChange::Strobe(sc)
}

fn wrap_color(sc: ColorStateChange) -> ControlMessagePayload {
    StateChange::Color(sc)
}

impl HandleOscStateChange<StateChange> for RotosphereQ3 {}
