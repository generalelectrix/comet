use crate::fixture::aquarius::{Aquarius, ControlMessage, StateChange};
use crate::fixture::prelude::*;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{GroupControlMap, HandleOscStateChange};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = Aquarius::NAME.0;

const LAMP_ON: Button = button(GROUP, "LampOn");

impl Aquarius {
    fn group(&self) -> &'static str {
        GROUP
    }

    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;
        LAMP_ON.map_state(map, LampOn);
        map.add_bipolar("Rotation", |v| Rotation(bipolar_fader_with_detent(v)));
    }
}

impl HandleOscStateChange<StateChange> for Aquarius {}
