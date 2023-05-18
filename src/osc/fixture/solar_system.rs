use rosc::OscMessage;

use crate::fixture::solar_system::SolarSystem;
use crate::fixture::solar_system::{StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::{HandleStateChange};
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

impl MapControls for SolarSystem {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::SolarSystem;
        use StateChange::*;
        map.add_bool(GROUP, "ShutterOpen", |v| SolarSystem(ShutterOpen(v)));
        map.add_bool(GROUP, "AutoShutter", |v| SolarSystem(AutoShutter(v)));
        map.add_radio_button_array(FRONT_GOBO_SELECT, |v| SolarSystem(FrontGobo(v)));
        map.add_bipolar(GROUP, "FrontRotation", |v| {
            SolarSystem(FrontRotation(bipolar_fader_with_detent(v)))
        });
        map.add_radio_button_array(REAR_GOBO_SELECT, |v| SolarSystem(RearGobo(v)));
        map.add_bipolar(GROUP, "RearRotation", |v| {
            SolarSystem(RearRotation(bipolar_fader_with_detent(v)))
        });
    }
}

impl HandleStateChange<StateChange> for SolarSystem {
    fn emit_state_change<S>(sc: StateChange, send: &mut S)
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
