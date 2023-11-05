use rosc::OscMessage;

use crate::fixture::solar_system::SolarSystem;
use crate::fixture::solar_system::StateChange;
use crate::fixture::FixtureControlMessage;
use crate::osc::HandleStateChange;

use crate::osc::basic_controls::button;
use crate::osc::basic_controls::Button;
use crate::osc::{ControlMap, MapControls, RadioButton};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "SolarSystem";

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

impl MapControls for SolarSystem {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::SolarSystem;
        use StateChange::*;
        SHUTTER_OPEN.map_state(map, |v| SolarSystem(ShutterOpen(v)));
        AUTO_SHUTTER.map_state(map, |v| SolarSystem(AutoShutter(v)));
        FRONT_GOBO_SELECT.map(map, |v| SolarSystem(FrontGobo(v)));
        map.add_bipolar(GROUP, "FrontRotation", |v| {
            SolarSystem(FrontRotation(bipolar_fader_with_detent(v)))
        });
        REAR_GOBO_SELECT.map(map, |v| SolarSystem(RearGobo(v)));
        map.add_bipolar(GROUP, "RearRotation", |v| {
            SolarSystem(RearRotation(bipolar_fader_with_detent(v)))
        });
    }
}

impl HandleStateChange<StateChange> for SolarSystem {
    fn emit_state_change<S>(sc: StateChange, send: &mut S, _talkback: crate::osc::TalkbackMode)
    where
        S: FnMut(OscMessage),
    {
        match sc {
            StateChange::FrontGobo(v) => FRONT_GOBO_SELECT.set(v, send),
            StateChange::RearGobo(v) => REAR_GOBO_SELECT.set(v, send),
            _ => (), // TODO: talkback for all controls
        }
    }
}
