use rosc::{OscMessage, OscType};

use super::ControlMap;
use crate::fixture::ControlMessage::{self as ShowControlMessage, H2O};
use crate::h2o::{FixedColor as FixedColorType, StateChange};
use crate::osc::ignore_payload;
use crate::util::bipolar_fader_with_detent;
use strum::IntoEnumIterator;

const GROUP: &str = "H2O";
const FIXED_COLOR: &str = "FixedColor";

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
    use StateChange::*;
    map.add_unipolar(GROUP, "Dimmer", |v| H2O(Dimmer(v)));
    map.add_bipolar(GROUP, "Rotation", |v| {
        H2O(Rotation(bipolar_fader_with_detent(v)))
    });
    map.add_enum_handler(GROUP, FIXED_COLOR, ignore_payload, |c, _| {
        H2O(StateChange::FixedColor(c))
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
            // Essentially a radio button implementation.
            for color_choice in FixedColorType::iter() {
                send(OscMessage {
                    addr: format!("/{}/{}/{}", GROUP, FIXED_COLOR, color_choice),
                    args: vec![OscType::Float(if color_choice == c { 1.0 } else { 0.0 })],
                });
            }
        }
        _ => (),
    }
}
