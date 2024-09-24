use rosc::OscMessage;

use super::generic::map_strobe;
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::wizard_extreme::{Color, StateChange, WizardExtreme};
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchAnimatedFixture;
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
    n: WizardExtreme::GOBO_COUNT,
    x_primary_coordinate: false,
};

const TWINKLE: Button = button(GROUP, "Twinkle");

const MIRROR_DRUM_ROTATION: Button = button(GROUP, "MirrorDrumRotation");
const MIRROR_DRUM_SWIVEL: Button = button(GROUP, "MirrorDrumSwivel");
const MIRROR_REFLECTOR_ROTATION: Button = button(GROUP, "MirrorReflectorRotation");

const ACTIVE: Button = button(GROUP, "Active");

impl EnumRadioButton for Color {}

impl MapControls for WizardExtreme {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        use StateChange::*;
        map.add_unipolar(GROUP, "Dimmer", |v| {
            ControlMessagePayload::fixture(Dimmer(v))
        });
        map_strobe(map, GROUP, "Strobe", &wrap_strobe);
        map.add_enum_handler(GROUP, COLOR, ignore_payload, |c, _| {
            ControlMessagePayload::fixture(Color(c))
        });
        TWINKLE.map_state(map, |v| ControlMessagePayload::fixture(Twinkle(v)));
        map.add_unipolar(GROUP, "TwinkleSpeed", |v| {
            ControlMessagePayload::fixture(TwinkleSpeed(v))
        });
        GOBO_SELECT.map(map, |v| ControlMessagePayload::fixture(Gobo(v)));
        map.add_bipolar(GROUP, "DrumRotation", |v| {
            ControlMessagePayload::fixture(DrumRotation(bipolar_fader_with_detent(v)))
        });
        MIRROR_DRUM_ROTATION.map_state(map, |v| {
            ControlMessagePayload::fixture(MirrorDrumRotation(v))
        });
        map.add_bipolar(GROUP, "DrumSwivel", |v| {
            ControlMessagePayload::fixture(DrumSwivel(v))
        });
        MIRROR_DRUM_SWIVEL.map_state(map, |v| ControlMessagePayload::fixture(MirrorDrumSwivel(v)));
        map.add_bipolar(GROUP, "ReflectorRotation", |v| {
            ControlMessagePayload::fixture(ReflectorRotation(bipolar_fader_with_detent(v)))
        });
        MIRROR_REFLECTOR_ROTATION.map_state(map, |v| {
            ControlMessagePayload::fixture(MirrorReflectorRotation(v))
        });
        ACTIVE.map_state(map, |v| ControlMessagePayload::fixture(Active(v)));
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        vec![(GROUP.to_string(), Self::NAME)]
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessagePayload {
    ControlMessagePayload::fixture(StateChange::Strobe(sc))
}

impl HandleStateChange<StateChange> for WizardExtreme {
    fn emit_state_change<S>(sc: StateChange, send: &mut S, _talkback: crate::osc::TalkbackMode)
    where
        S: crate::osc::EmitControlMessage,
    {
        match sc {
            StateChange::Color(c) => {
                c.set(GROUP, COLOR, send);
            }
            StateChange::Gobo(v) => GOBO_SELECT.set(v, send),
            StateChange::MirrorDrumRotation(v) => MIRROR_DRUM_ROTATION.send(v, send),
            StateChange::MirrorReflectorRotation(v) => MIRROR_REFLECTOR_ROTATION.send(v, send),
            StateChange::MirrorDrumSwivel(v) => MIRROR_DRUM_SWIVEL.send(v, send),
            StateChange::Active(v) => ACTIVE.send(v, send),
            _ => (),
        }
    }
}
