use log::error;
use rosc::OscMessage;

use super::{get_bool, ControlMap, RadioButton};
use crate::fixture::ControlMessage::{self as ShowControlMessage, Swarmolon};
use crate::generic::GenericStrobeStateChange;
use crate::osc::generic::map_strobe;
use crate::swarmolon::{ControlMessage, DerbyColor, StateChange, WhiteStrobeStateChange};

const GROUP: &str = "Swarmolon";

const STROBE_PROGRAM_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "WhiteStrobeProgram",
    n: 10,
    x_primary_coordinate: false,
};

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
    use ControlMessage::*;
    use StateChange::*;
    map.add_enum_handler(GROUP, "DerbyColor", get_bool, |c, v| {
        Swarmolon(Set(DerbyColor(c, v)))
    });
    map_strobe(map, GROUP, "DerbyStrobe", &wrap_derby_strobe);
    map.add_bipolar(GROUP, "DerbyRotation", |v| Swarmolon(Set(DerbyRotation(v))));
    map_strobe(map, GROUP, "WhiteStrobe", &wrap_white_strobe);
    map.add_radio_button_array(STROBE_PROGRAM_SELECT, |v| {
        Swarmolon(Set(WhiteStrobe(WhiteStrobeStateChange::Program(v))))
    });

    map.add_bool(GROUP, "RedLaserOn", |v| Swarmolon(Set(RedLaserOn(v))));
    map.add_bool(GROUP, "GreenLaserOn", |v| Swarmolon(Set(GreenLaserOn(v))));
    map_strobe(map, GROUP, "LaserStrobe", &wrap_laser_strobe);
    map.add_bipolar(GROUP, "LaserRotation", |v| Swarmolon(Set(LaserRotation(v))));

    // "Global" strobe rate control, for simpler one-fader control.
    // This is a bit of a hack, since it has no talkback channel.
    // This will need to be refactored if we want to use uniform talkback.
    map.add_unipolar(GROUP, "StrobeRate", |v| Swarmolon(StrobeRate(v)));
}

fn wrap_derby_strobe(sc: GenericStrobeStateChange) -> ShowControlMessage {
    Swarmolon(ControlMessage::Set(StateChange::DerbyStrobe(sc)))
}

fn wrap_white_strobe(sc: GenericStrobeStateChange) -> ShowControlMessage {
    Swarmolon(ControlMessage::Set(StateChange::WhiteStrobe(
        WhiteStrobeStateChange::State(sc),
    )))
}

fn wrap_laser_strobe(sc: GenericStrobeStateChange) -> ShowControlMessage {
    Swarmolon(ControlMessage::Set(StateChange::LaserStrobe(sc)))
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
