use crate::fixture::aquarius::{Aquarius, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{ControlMap, HandleStateChange, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "Aquarius";

const LAMP_ON: Button = button(GROUP, "LampOn");

impl MapControls for Aquarius {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::Aquarius;
        use StateChange::*;
        LAMP_ON.map_state(map, |v| Aquarius(LampOn(v)));
        map.add_bipolar(GROUP, "Rotation", |v| {
            Aquarius(Rotation(bipolar_fader_with_detent(v)))
        });
    }
}

impl HandleStateChange<StateChange> for Aquarius {}
