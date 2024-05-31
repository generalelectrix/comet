use rosc::OscMessage;

use super::generic::map_strobe;
use crate::fixture::astroscan::{Astroscan, Color, StateChange};
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::FixtureControlMessage;
use crate::osc::basic_controls::{button, Button};
use crate::osc::radio_button::EnumRadioButton;
use crate::osc::{ignore_payload, HandleStateChange};
use crate::osc::{ControlMap, MapControls, RadioButton};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "Astroscan";
const COLOR: &str = "Color";

const GOBO_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Gobo",
    n: Astroscan::GOBO_COUNT,
    x_primary_coordinate: false,
};

const LAMP_ON: Button = button(GROUP, "LampOn");

impl EnumRadioButton for Color {}

impl MapControls for Astroscan {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::Astroscan;
        use StateChange::*;
        LAMP_ON.map_state(map, |v| Astroscan(LampOn(v)));
        map.add_unipolar(GROUP, "Dimmer", |v| Astroscan(Dimmer(v)));
        map_strobe(map, GROUP, "Strobe", &wrap_strobe);
        map.add_enum_handler(GROUP, COLOR, ignore_payload, |c, _| Astroscan(Color(c)));
        map.add_unipolar(GROUP, "Iris", |v| Astroscan(Iris(v)));
        GOBO_SELECT.map(map, |v| Astroscan(Gobo(v)));
        map.add_bipolar(GROUP, "GoboRotation", |v| {
            Astroscan(GoboRotation(bipolar_fader_with_detent(v)))
        });
        map.add_bipolar(GROUP, "MirrorRotation", |v| {
            Astroscan(MirrorRotation(bipolar_fader_with_detent(v)))
        });
        map.add_bipolar(GROUP, "Pan", |v| {
            Astroscan(Pan(bipolar_fader_with_detent(v)))
        });
        map.add_bipolar(GROUP, "Tilt", |v| {
            Astroscan(Tilt(bipolar_fader_with_detent(v)))
        });
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> FixtureControlMessage {
    FixtureControlMessage::Astroscan(StateChange::Strobe(sc))
}

impl HandleStateChange<StateChange> for Astroscan {
    fn emit_state_change<S>(sc: StateChange, send: &mut S, _talkback: crate::osc::TalkbackMode)
    where
        S: FnMut(OscMessage),
    {
        match sc {
            StateChange::LampOn(v) => LAMP_ON.send(v, send),
            StateChange::Color(c) => {
                c.set(GROUP, COLOR, send);
            }
            StateChange::Gobo(v) => GOBO_SELECT.set(v, send),
            _ => (),
        }
    }
}
