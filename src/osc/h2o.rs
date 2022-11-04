use rosc::OscMessage;

use super::{update_enum_radio_select, ControlMap};
use crate::fixture::ControlMessage::{self as ShowControlMessage, H2O};
use crate::h2o::StateChange;
use crate::osc::ignore_payload;
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "H2O";
const FIXED_COLOR: &str = "FixedColor";

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
    use StateChange::*;
    map.add_unipolar(GROUP, "Dimmer", |v| H2O(Dimmer(v)));
    map.add_bipolar(GROUP, "Rotation", |v| {
        H2O(Rotation(bipolar_fader_with_detent(v)))
    });
    map.add_enum_handler(GROUP, FIXED_COLOR, ignore_payload, |c, _| {
        H2O(FixedColor(c))
    });
    map.add_bool(GROUP, "ColorRotate", |v| H2O(ColorRotate(v)));
    map.add_bipolar(GROUP, "ColorRotation", |v| {
        H2O(ColorRotation(bipolar_fader_with_detent(v)))
    });
}

pub fn handle_state_change<S>(sc: StateChange, send: &mut S)
where
    S: FnMut(OscMessage),
{
    match sc {
        StateChange::FixedColor(c) => {
            update_enum_radio_select(GROUP, FIXED_COLOR, &c, send);
        }
        _ => (),
    }
}
