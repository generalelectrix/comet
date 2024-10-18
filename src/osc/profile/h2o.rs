use crate::fixture::h2o::{FixedColor, StateChange, H2O};
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchAnimatedFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::radio_button::EnumRadioButton;
use crate::osc::{ignore_payload, HandleOscStateChange};
use crate::osc::{GroupControlMap, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "H2O";
const FIXED_COLOR: &str = "FixedColor";

const COLOR_ROTATE: Button = button(GROUP, "ColorRotate");

impl EnumRadioButton for FixedColor {}

impl MapControls for H2O {
    fn group(&self) -> &'static str {
        GROUP
    }

    fn map_controls(&self, map: &mut GroupControlMap<ControlMessagePayload>) {
        use StateChange::*;
        map.add_unipolar("Dimmer", |v| ControlMessagePayload::fixture(Dimmer(v)));
        map.add_bipolar("Rotation", |v| {
            ControlMessagePayload::fixture(Rotation(bipolar_fader_with_detent(v)))
        });
        map.add_enum_handler(FIXED_COLOR, ignore_payload, |c, _| {
            ControlMessagePayload::fixture(FixedColor(c))
        });
        COLOR_ROTATE.map_state(map, |v| ControlMessagePayload::fixture(ColorRotate(v)));
        map.add_bipolar("ColorRotation", |v| {
            ControlMessagePayload::fixture(ColorRotation(bipolar_fader_with_detent(v)))
        });
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        vec![(GROUP.to_string(), Self::NAME)]
    }
}

impl HandleOscStateChange<StateChange> for H2O {
    fn emit_osc_state_change<S>(sc: StateChange, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
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
