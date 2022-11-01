use log::error;
use rosc::OscMessage;

use super::{get_bool, ControlMap, RadioButton};
use crate::fixture::ControlMessage::{self as ShowControlMessage, Swarmolon};
use crate::generic::GenericStrobeStateChange;
use crate::osc::generic::map_strobe;
use crate::swarmolon::{DerbyColor, StateChange, WhiteStrobeStateChange};

const GROUP: &str = "Swarmolon";

const STROBE_PROGRAM_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "WhiteStrobeProgram",
    n: 10,
};

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
    use StateChange::*;
    map.add_enum_handler(GROUP, "DerbyColor", get_bool, |c, v| {
        Swarmolon(DerbyColor(c, v))
    });
    map_strobe(map, GROUP, "DerbyStrobe", &wrap_derby_strobe);
    map_strobe(map, GROUP, "WhiteStrobe", &wrap_white_strobe);
    map.add_radio_button_array(STROBE_PROGRAM_SELECT, |v| {
        Swarmolon(WhiteStrobe(WhiteStrobeStateChange::Program(v)))
    });
}

fn wrap_derby_strobe(sc: GenericStrobeStateChange) -> ShowControlMessage {
    Swarmolon(StateChange::DerbyStrobe(sc))
}

fn wrap_white_strobe(sc: GenericStrobeStateChange) -> ShowControlMessage {
    Swarmolon(StateChange::WhiteStrobe(WhiteStrobeStateChange::State(sc)))
}

pub fn handle_state_change<S>(sc: StateChange, send: &mut S)
where
    S: FnMut(OscMessage),
{
    use StateChange::*;
    match sc {
        WhiteStrobe(WhiteStrobeStateChange::Program(v)) => {
            if let Err(e) = STROBE_PROGRAM_SELECT.set(v, send) {
                error!("Swarmolon strobe program select update error: {}.", e);
            }
        }
        _ => (),
    }
}
