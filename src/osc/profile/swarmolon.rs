use super::generic::map_strobe;
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::swarmolon::{
    ControlMessage, DerbyColor, StateChange, Swarmolon, WhiteStrobeStateChange,
};

use crate::fixture::PatchFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::radio_button::EnumRadioButton;
use crate::osc::{get_bool, GroupControlMap, HandleOscStateChange, RadioButton};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = Swarmolon::NAME.0;

const STROBE_PROGRAM_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "WhiteStrobeProgram",
    n: 10,
    x_primary_coordinate: false,
};

const RED_LASER_ON: Button = button(GROUP, "RedLaserOn");
const GREEN_LASER_ON: Button = button(GROUP, "GreenLaserOn");

impl EnumRadioButton for DerbyColor {}

impl Swarmolon {
    fn group(&self) -> &'static str {
        GROUP
    }
    fn map_controls(&self, map: &mut GroupControlMap<ControlMessage>) {
        use ControlMessage::*;
        use StateChange::*;

        map.add_enum_handler("DerbyColor", get_bool, |c, v| Set(DerbyColor(c, v)));
        map_strobe(map, "DerbyStrobe", &wrap_derby_strobe);
        map.add_bipolar("DerbyRotation", |v| {
            Set(DerbyRotation(bipolar_fader_with_detent(v)))
        });
        map_strobe(map, "WhiteStrobe", &wrap_white_strobe);
        STROBE_PROGRAM_SELECT.map(map, |v| {
            Set(WhiteStrobe(WhiteStrobeStateChange::Program(v)))
        });

        RED_LASER_ON.map_state(map, |v| Set(RedLaserOn(v)));
        GREEN_LASER_ON.map_state(map, |v| Set(GreenLaserOn(v)));
        map_strobe(map, "LaserStrobe", &wrap_laser_strobe);
        map.add_bipolar("LaserRotation", |v| {
            Set(LaserRotation(bipolar_fader_with_detent(v)))
        });

        // "Global" strobe rate control, for simpler one-fader control.
        // This is a bit of a hack, since it has no talkback channel.
        // This will need to be refactored if we want to use uniform talkback.
        map.add_unipolar("StrobeRate", |v| StrobeRate(v));
    }
}

fn wrap_derby_strobe(sc: GenericStrobeStateChange) -> ControlMessage {
    ControlMessage::Set(StateChange::DerbyStrobe(sc))
}

fn wrap_white_strobe(sc: GenericStrobeStateChange) -> ControlMessage {
    ControlMessage::Set(StateChange::WhiteStrobe(WhiteStrobeStateChange::State(sc)))
}

fn wrap_laser_strobe(sc: GenericStrobeStateChange) -> ControlMessage {
    ControlMessage::Set(StateChange::LaserStrobe(sc))
}

impl HandleOscStateChange<StateChange> for Swarmolon {
    fn emit_osc_state_change<S>(sc: StateChange, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        use StateChange::*;
        #[allow(clippy::single_match)]
        match sc {
            WhiteStrobe(WhiteStrobeStateChange::Program(v)) => STROBE_PROGRAM_SELECT.set(v, send),
            _ => (),
        }
    }
}
