use log::error;
use rosc::OscMessage;

use super::generic::map_strobe;
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::rush_wizard::{Color, RushWizard, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::radio_button::EnumRadioButton;
use crate::osc::{ignore_payload, HandleStateChange};
use crate::osc::{ControlMap, MapControls, RadioButton};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "RushWizard";
const COLOR: &str = "Color";

const GOBO_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Gobo",
    n: 16,
    x_primary_coordinate: false,
};

impl EnumRadioButton for Color {}

impl MapControls for RushWizard {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::RushWizard;
        use StateChange::*;
        map.add_unipolar(GROUP, "Dimmer", |v| RushWizard(Dimmer(v)));
        map_strobe(map, GROUP, "Strobe", &wrap_strobe);
        map.add_enum_handler(GROUP, COLOR, ignore_payload, |c, _| RushWizard(Color(c)));
        map.add_bool(GROUP, "Twinkle", |v| RushWizard(Twinkle(v)));
        map.add_unipolar(GROUP, "TwinkleSpeed", |v| RushWizard(TwinkleSpeed(v)));
        map.add_radio_button_array(GOBO_SELECT, |v| RushWizard(Gobo(v)));
        map.add_bipolar(GROUP, "DrumRotation", |v| {
            RushWizard(DrumRotation(bipolar_fader_with_detent(v)))
        });
        map.add_bipolar(GROUP, "DrumSwivel", |v| RushWizard(DrumSwivel(v)));
        map.add_bipolar(GROUP, "ReflectorRotation", |v| {
            RushWizard(ReflectorRotation(bipolar_fader_with_detent(v)))
        });
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> FixtureControlMessage {
    FixtureControlMessage::RushWizard(StateChange::Strobe(sc))
}

impl HandleStateChange<StateChange> for RushWizard {
    fn emit_state_change<S>(sc: StateChange, send: &mut S)
    where
        S: FnMut(OscMessage),
    {
        match sc {
            StateChange::Color(c) => {
                c.set(GROUP, COLOR, send);
            }
            StateChange::Gobo(v) => GOBO_SELECT.set(v, send),
            _ => (),
        }
    }
}
