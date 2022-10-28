use rosc::OscMessage;

use super::ControlMap;
use crate::aquarius::StateChange;
use crate::fixture::ControlMessage::{self as ShowControlMessage, Aquarius};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "Aquarius";

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
    use StateChange::*;
    map.add_bool(GROUP, "LampOn", |v| Aquarius(LampOn(v)));
    map.add_bipolar(GROUP, "Rotation", |v| {
        Aquarius(Rotation(bipolar_fader_with_detent(v)))
    });
}

pub fn handle_state_change<S>(_: StateChange, _: &mut S)
where
    S: FnMut(OscMessage),
{
    // No controls with talkback.
}
