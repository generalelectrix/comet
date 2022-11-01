use rosc::OscMessage;

use super::{get_bool, ControlMap};
use crate::fixture::ControlMessage::{self as ShowControlMessage, Swarmolon};
use crate::swarmolon::{DerbyColor, StateChange};

const GROUP: &str = "Swarmolon";

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
    use StateChange::*;
    map.add_enum_handler(GROUP, "DerbyColor", get_bool, |c, v| {
        Swarmolon(DerbyColor(c, v))
    });
}

pub fn handle_state_change<S>(_: StateChange, _: &mut S)
where
    S: FnMut(OscMessage),
{
    // No controls with talkback.
}
