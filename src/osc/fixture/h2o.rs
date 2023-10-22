use rosc::OscMessage;

use crate::fixture::h2o::{FixedColor, StateChange, H2O};
use crate::fixture::FixtureControlMessage;
use crate::osc::basic_controls::{button, Button};
use crate::osc::radio_button::EnumRadioButton;
use crate::osc::{ignore_payload, HandleStateChange};
use crate::osc::{ControlMap, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "H2O";
const FIXED_COLOR: &str = "FixedColor";

const COLOR_ROTATE: Button = button(GROUP, "ColorRotate");

impl EnumRadioButton for FixedColor {}

impl MapControls for H2O {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::H2O;
        use StateChange::*;
        map.add_unipolar(GROUP, "Dimmer", |v| H2O(Dimmer(v)));
        map.add_bipolar(GROUP, "Rotation", |v| {
            H2O(Rotation(bipolar_fader_with_detent(v)))
        });
        map.add_enum_handler(GROUP, FIXED_COLOR, ignore_payload, |c, _| {
            H2O(FixedColor(c))
        });
        COLOR_ROTATE.map_state(map, |v| H2O(ColorRotate(v)));
        map.add_bipolar(GROUP, "ColorRotation", |v| {
            H2O(ColorRotation(bipolar_fader_with_detent(v)))
        });
    }
}

impl HandleStateChange<StateChange> for H2O {
    fn emit_state_change<S>(sc: StateChange, send: &mut S, talkback: crate::osc::TalkbackMode)
    where
        S: FnMut(OscMessage),
    {
        #[allow(clippy::single_match)]
        match sc {
            StateChange::FixedColor(c) => {
                c.set(GROUP, FIXED_COLOR, send);
            }
            _ => (),
        }
    }
}
