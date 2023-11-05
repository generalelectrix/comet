use rosc::OscMessage;

use super::generic::map_strobe;
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::wizard_extreme::{Color, StateChange, WizardExtreme};
use crate::fixture::FixtureControlMessage;
use crate::osc::basic_controls::{button, Button};
use crate::osc::radio_button::EnumRadioButton;
use crate::osc::{ignore_payload, HandleStateChange};
use crate::osc::{ControlMap, MapControls, RadioButton};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "WizardExtreme";
const COLOR: &str = "Color";

const GOBO_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Gobo",
    n: 14,
    x_primary_coordinate: false,
};

const TWINKLE: Button = button(GROUP, "Twinkle");

impl EnumRadioButton for Color {}

impl MapControls for WizardExtreme {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::WizardExtreme;
        use StateChange::*;
        map.add_unipolar(GROUP, "Dimmer", |v| WizardExtreme(Dimmer(v)));
        map_strobe(map, GROUP, "Strobe", &wrap_strobe);
        map.add_enum_handler(GROUP, COLOR, ignore_payload, |c, _| WizardExtreme(Color(c)));
        TWINKLE.map_state(map, |v| WizardExtreme(Twinkle(v)));
        map.add_unipolar(GROUP, "TwinkleSpeed", |v| WizardExtreme(TwinkleSpeed(v)));
        GOBO_SELECT.map(map, |v| WizardExtreme(Gobo(v)));
        map.add_bipolar(GROUP, "DrumRotation", |v| {
            WizardExtreme(DrumRotation(bipolar_fader_with_detent(v)))
        });
        map.add_bipolar(GROUP, "DrumSwivel", |v| WizardExtreme(DrumSwivel(v)));
        map.add_bipolar(GROUP, "ReflectorRotation", |v| {
            WizardExtreme(ReflectorRotation(bipolar_fader_with_detent(v)))
        });
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> FixtureControlMessage {
    FixtureControlMessage::WizardExtreme(StateChange::Strobe(sc))
}

impl HandleStateChange<StateChange> for WizardExtreme {
    fn emit_state_change<S>(sc: StateChange, send: &mut S, _talkback: crate::osc::TalkbackMode)
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
