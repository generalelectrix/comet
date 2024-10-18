use crate::fixture::h2o::{FixedColor, StateChange, H2O};

use crate::fixture::PatchAnimatedFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::radio_button::EnumRadioButton;
use crate::osc::{ignore_payload, HandleOscStateChange};
use crate::osc::{GroupControlMap};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = H2O::NAME.0;
const FIXED_COLOR: &str = "FixedColor";

const COLOR_ROTATE: Button = button(GROUP, "ColorRotate");

impl EnumRadioButton for FixedColor {}

impl H2O {
    fn group(&self) -> &'static str {
        GROUP
    }

    fn map_controls(&self, map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;
        map.add_unipolar("Dimmer", |v| Dimmer(v));
        map.add_bipolar("Rotation", |v| {
            Rotation(bipolar_fader_with_detent(v))
        });
        map.add_enum_handler(FIXED_COLOR, ignore_payload, |c, _| {
            FixedColor(c)
        });
        COLOR_ROTATE.map_state(map, |v| ColorRotate(v));
        map.add_bipolar("ColorRotation", |v| {
            ColorRotation(bipolar_fader_with_detent(v))
        });
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
