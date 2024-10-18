use crate::fixture::solar_system::SolarSystem;
use crate::fixture::solar_system::{ControlMessage, StateChange};

use crate::osc::HandleOscStateChange;

use crate::fixture::PatchAnimatedFixture;
use crate::osc::basic_controls::button;
use crate::osc::basic_controls::Button;
use crate::osc::{GroupControlMap, RadioButton};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = SolarSystem::NAME.0;

const FRONT_GOBO_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "FrontGobo",
    n: SolarSystem::GOBO_COUNT,
    x_primary_coordinate: false,
};

const REAR_GOBO_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "RearGobo",
    n: SolarSystem::GOBO_COUNT,
    x_primary_coordinate: false,
};

const SHUTTER_OPEN: Button = button(GROUP, "ShutterOpen");
const AUTO_SHUTTER: Button = button(GROUP, "AutoShutter");

impl SolarSystem {
    fn group(&self) -> &'static str {
        GROUP
    }
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;
        SHUTTER_OPEN.map_state(map, |v| ShutterOpen(v));
        AUTO_SHUTTER.map_state(map, |v| AutoShutter(v));
        FRONT_GOBO_SELECT.map(map, |v| FrontGobo(v));
        map.add_bipolar("FrontRotation", |v| {
            FrontRotation(bipolar_fader_with_detent(v))
        });
        REAR_GOBO_SELECT.map(map, |v| RearGobo(v));
        map.add_bipolar("RearRotation", |v| {
            RearRotation(bipolar_fader_with_detent(v))
        });
    }
}

impl HandleOscStateChange<StateChange> for SolarSystem {
    fn emit_osc_state_change<S>(sc: StateChange, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        match sc {
            StateChange::FrontGobo(v) => FRONT_GOBO_SELECT.set(v, send),
            StateChange::RearGobo(v) => REAR_GOBO_SELECT.set(v, send),
            _ => (), // TODO: talkback for all controls
        }
    }
}
