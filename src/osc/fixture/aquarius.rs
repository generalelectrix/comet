use crate::fixture::aquarius::{Aquarius, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::{ControlMap, HandleStateChange, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "Aquarius";

impl MapControls for Aquarius {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::Aquarius;
        use StateChange::*;
        map.add_bool(GROUP, "LampOn", |v| Aquarius(LampOn(v)));
        map.add_bipolar(GROUP, "Rotation", |v| {
            Aquarius(Rotation(bipolar_fader_with_detent(v)))
        });
    }
}

impl HandleStateChange<StateChange> for Aquarius {}
