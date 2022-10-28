use rosc::OscMessage;

use super::ControlMap;
use crate::fixture::ControlMessage::{self as ShowControlMessage, Radiance};
use crate::radiance::StateChange;

const GROUP: &str = "Radiance";

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
    use StateChange::*;
    map.add_unipolar(GROUP, "Haze", |v| Radiance(Haze(v)));
    map.add_unipolar(GROUP, "Fan", |v| Radiance(Fan(v)));
}

pub fn handle_state_change<S>(_: StateChange, _: &mut S)
where
    S: FnMut(OscMessage),
{
    // No controls with talkback.
}
