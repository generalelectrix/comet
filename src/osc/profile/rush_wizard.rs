use super::generic::map_strobe;
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::rush_wizard::{Color, RushWizard, StateChange};
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::radio_button::EnumRadioButton;
use crate::osc::{ignore_payload, HandleOscStateChange};
use crate::osc::{GroupControlMap, MapControls, RadioButton};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = RushWizard::NAME.0;
const COLOR: &str = "Color";

const GOBO_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Gobo",
    n: 16,
    x_primary_coordinate: false,
};

const TWINKLE: Button = button(GROUP, "Twinkle");

impl EnumRadioButton for Color {}

impl MapControls for RushWizard {
    fn group(&self) -> &'static str {
        GROUP
    }
    fn map_controls(&self, map: &mut GroupControlMap<ControlMessagePayload>) {
        use StateChange::*;
        map.add_unipolar("Dimmer", |v| ControlMessagePayload::fixture(Dimmer(v)));
        map_strobe(map, "Strobe", &wrap_strobe);
        map.add_enum_handler(COLOR, ignore_payload, |c, _| {
            ControlMessagePayload::fixture(Color(c))
        });
        TWINKLE.map_state(map, |v| ControlMessagePayload::fixture(Twinkle(v)));
        map.add_unipolar("TwinkleSpeed", |v| {
            ControlMessagePayload::fixture(TwinkleSpeed(v))
        });
        GOBO_SELECT.map(map, |v| ControlMessagePayload::fixture(Gobo(v)));
        map.add_bipolar("DrumRotation", |v| {
            ControlMessagePayload::fixture(DrumRotation(bipolar_fader_with_detent(v)))
        });
        map.add_bipolar("DrumSwivel", |v| {
            ControlMessagePayload::fixture(DrumSwivel(v))
        });
        map.add_bipolar("ReflectorRotation", |v| {
            ControlMessagePayload::fixture(ReflectorRotation(bipolar_fader_with_detent(v)))
        });
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessagePayload {
    ControlMessagePayload::fixture(StateChange::Strobe(sc))
}

impl HandleOscStateChange<StateChange> for RushWizard {
    fn emit_osc_state_change<S>(sc: StateChange, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
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
