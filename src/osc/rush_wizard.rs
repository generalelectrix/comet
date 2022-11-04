use log::error;
use rosc::OscMessage;

use super::{update_enum_radio_select, ControlMap, RadioButton};
use crate::fixture::ControlMessage::{self as ShowControlMessage, RushWizard};
use crate::generic::GenericStrobeStateChange;
use crate::osc::generic::map_strobe;
use crate::osc::ignore_payload;
use crate::rush_wizard::StateChange;
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "RushWizard";
const COLOR: &str = "Color";

const GOBO_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Gobo",
    n: 16,
    x_primary_coordinate: false,
};

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
    use StateChange::*;
    map.add_unipolar(GROUP, "Dimmer", |v| RushWizard(Dimmer(v)));
    map_strobe(map, GROUP, "Strobe", &wrap_strobe);
    map.add_enum_handler(GROUP, COLOR, ignore_payload, |c, _| RushWizard(Color(c)));
    map.add_bool(GROUP, "Twinkle", |v| RushWizard(Twinkle(v)));
    map.add_unipolar(GROUP, "TwinkleSpeed", |v| RushWizard(TwinkleSpeed(v)));
    map.add_radio_button_array(GOBO_SELECT, |v| RushWizard(Gobo(v)));
    map.add_bipolar(GROUP, "DrumRotation", |v| {
        RushWizard(DrumRotation(bipolar_fader_with_detent(v)))
    });
    map.add_bipolar(GROUP, "DrumSwivel", |v| RushWizard(DrumSwivel(v)));
    map.add_bipolar(GROUP, "ReflectorRotation", |v| {
        RushWizard(ReflectorRotation(bipolar_fader_with_detent(v)))
    });
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ShowControlMessage {
    RushWizard(StateChange::Strobe(sc))
}

pub fn handle_state_change<S>(sc: StateChange, send: &mut S)
where
    S: FnMut(OscMessage),
{
    match sc {
        StateChange::Color(c) => {
            update_enum_radio_select(GROUP, COLOR, &c, send);
        }
        StateChange::Gobo(v) => {
            if let Err(e) = GOBO_SELECT.set(v, send) {
                error!("Rush Wizard gobo select update error: {}.", e);
            }
        }
        _ => (),
    }
}
