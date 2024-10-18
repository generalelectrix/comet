use super::generic::map_strobe;
use crate::fixture::astroscan::{Astroscan, Color, ControlMessage, StateChange};
use crate::fixture::generic::GenericStrobeStateChange;

use crate::fixture::PatchAnimatedFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::radio_button::EnumRadioButton;
use crate::osc::{ignore_payload, HandleOscStateChange};
use crate::osc::{GroupControlMap, RadioButton};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = Astroscan::NAME.0;
const COLOR: &str = "Color";

const GOBO_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Gobo",
    n: Astroscan::GOBO_COUNT,
    x_primary_coordinate: false,
};

const LAMP_ON: Button = button(GROUP, "LampOn");

const MIRROR_GOBO_ROTATION: Button = button(GROUP, "MirrorGoboRotation");
const MIRROR_MIRROR_ROTATION: Button = button(GROUP, "MirrorMirrorRotation");
const MIRROR_PAN: Button = button(GROUP, "MirrorPan");
const MIRROR_TILT: Button = button(GROUP, "MirrorTilt");

const ACTIVE: Button = button(GROUP, "Active");

impl EnumRadioButton for Color {}

impl Astroscan {
    fn group(&self) -> &'static str {
        GROUP
    }

    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;
        LAMP_ON.map_state(map, LampOn);
        map.add_unipolar("Dimmer", Dimmer);
        map_strobe(map, "Strobe", &wrap_strobe);
        map.add_enum_handler(COLOR, ignore_payload, |c, _| Color(c));
        map.add_unipolar("Iris", Iris);
        GOBO_SELECT.map(map, Gobo);
        map.add_bipolar("GoboRotation", |v| {
            GoboRotation(bipolar_fader_with_detent(v))
        });
        MIRROR_GOBO_ROTATION.map_state(map, MirrorGoboRotation);
        map.add_bipolar("MirrorRotation", |v| {
            MirrorRotation(bipolar_fader_with_detent(v))
        });
        MIRROR_MIRROR_ROTATION.map_state(map, MirrorMirrorRotation);
        map.add_bipolar("Pan", |v| Pan(bipolar_fader_with_detent(v)));
        MIRROR_PAN.map_state(map, MirrorPan);
        map.add_bipolar("Tilt", |v| Tilt(bipolar_fader_with_detent(v)));
        MIRROR_TILT.map_state(map, MirrorTilt);
        ACTIVE.map_state(map, Active);
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessage {
    StateChange::Strobe(sc)
}

impl HandleOscStateChange<StateChange> for Astroscan {
    fn emit_osc_state_change<S>(sc: StateChange, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        match sc {
            StateChange::LampOn(v) => LAMP_ON.send(v, send),
            StateChange::MirrorGoboRotation(v) => MIRROR_GOBO_ROTATION.send(v, send),
            StateChange::MirrorMirrorRotation(v) => MIRROR_MIRROR_ROTATION.send(v, send),
            StateChange::MirrorPan(v) => MIRROR_PAN.send(v, send),
            StateChange::MirrorTilt(v) => MIRROR_TILT.send(v, send),
            StateChange::Active(v) => ACTIVE.send(v, send),
            StateChange::Color(c) => {
                c.set(GROUP, COLOR, send);
            }
            StateChange::Gobo(v) => GOBO_SELECT.set(v, send),
            _ => (),
        }
    }
}
