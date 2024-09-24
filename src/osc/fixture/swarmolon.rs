use rosc::OscMessage;

use super::generic::map_strobe;
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::swarmolon::{
    ControlMessage, DerbyColor, StateChange, Swarmolon, WhiteStrobeStateChange,
};
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::radio_button::EnumRadioButton;
use crate::osc::{get_bool, ControlMap, HandleOscStateChange, MapControls, RadioButton};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "Swarmolon";

const STROBE_PROGRAM_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "WhiteStrobeProgram",
    n: 10,
    x_primary_coordinate: false,
};

const RED_LASER_ON: Button = button(GROUP, "RedLaserOn");
const GREEN_LASER_ON: Button = button(GROUP, "GreenLaserOn");

impl EnumRadioButton for DerbyColor {}

impl MapControls for Swarmolon {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        use ControlMessage::*;
        use StateChange::*;

        map.add_enum_handler(GROUP, "DerbyColor", get_bool, |c, v| {
            ControlMessagePayload::fixture(Set(DerbyColor(c, v)))
        });
        map_strobe(map, GROUP, "DerbyStrobe", &wrap_derby_strobe);
        map.add_bipolar(GROUP, "DerbyRotation", |v| {
            ControlMessagePayload::fixture(Set(DerbyRotation(bipolar_fader_with_detent(v))))
        });
        map_strobe(map, GROUP, "WhiteStrobe", &wrap_white_strobe);
        STROBE_PROGRAM_SELECT.map(map, |v| {
            ControlMessagePayload::fixture(Set(WhiteStrobe(WhiteStrobeStateChange::Program(v))))
        });

        RED_LASER_ON.map_state(map, |v| ControlMessagePayload::fixture(Set(RedLaserOn(v))));
        GREEN_LASER_ON.map_state(map, |v| {
            ControlMessagePayload::fixture(Set(GreenLaserOn(v)))
        });
        map_strobe(map, GROUP, "LaserStrobe", &wrap_laser_strobe);
        map.add_bipolar(GROUP, "LaserRotation", |v| {
            ControlMessagePayload::fixture(Set(LaserRotation(bipolar_fader_with_detent(v))))
        });

        // "Global" strobe rate control, for simpler one-fader control.
        // This is a bit of a hack, since it has no talkback channel.
        // This will need to be refactored if we want to use uniform talkback.
        map.add_unipolar(GROUP, "StrobeRate", |v| {
            ControlMessagePayload::fixture(StrobeRate(v))
        });
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        vec![(GROUP.to_string(), Self::NAME)]
    }
}

fn wrap_derby_strobe(sc: GenericStrobeStateChange) -> ControlMessagePayload {
    ControlMessagePayload::fixture(ControlMessage::Set(StateChange::DerbyStrobe(sc)))
}

fn wrap_white_strobe(sc: GenericStrobeStateChange) -> ControlMessagePayload {
    ControlMessagePayload::fixture(ControlMessage::Set(StateChange::WhiteStrobe(
        WhiteStrobeStateChange::State(sc),
    )))
}

fn wrap_laser_strobe(sc: GenericStrobeStateChange) -> ControlMessagePayload {
    ControlMessagePayload::fixture(ControlMessage::Set(StateChange::LaserStrobe(sc)))
}

impl HandleOscStateChange<StateChange> for Swarmolon {
    fn emit_osc_state_change<S>(sc: StateChange, send: &mut S, _talkback: crate::osc::TalkbackMode)
    where
        S: crate::osc::EmitOscMessage,
    {
        use StateChange::*;
        #[allow(clippy::single_match)]
        match sc {
            WhiteStrobe(WhiteStrobeStateChange::Program(v)) => STROBE_PROGRAM_SELECT.set(v, send),
            _ => (),
        }
    }
}
