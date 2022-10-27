use super::{ControlMap, RadioButton};
use crate::fixture::ControlMessage::{self as ShowControlMessage, Comet};
use crate::{
    comet::{ControlMessage, StateChange, Step as Direction},
    osc::quadratic,
};
use log::error;
use rosc::OscMessage;
// Control group names.
const CONTROLS: &str = "Controls";
const MUSIC: &str = "Music";
const DEBUG: &str = "Debug";

const MACRO_SELECT_RADIO_BUTTON: RadioButton = RadioButton {
    group: CONTROLS,
    control: "SelectMacro",
    n: 10,
};

pub fn map_comet_controls(map: &mut ControlMap<ShowControlMessage>) {
    use ControlMessage::*;
    use StateChange::*;
    map.add_bool(CONTROLS, "Shutter", |v| Comet(Set(Shutter(v))));
    map.add_bool(CONTROLS, "Strobe", |v| Comet(Set(Strobe(v))));
    map.add_unipolar(CONTROLS, "StrobeRate", |v| {
        Comet(Set(StrobeRate(quadratic(v))))
    });
    map.add_unipolar(CONTROLS, "Mspeed", |v| Comet(Set(MirrorSpeed(v))));
    map.add_bool(CONTROLS, "AutoStep", |v| Comet(Set(AutoStep(v))));
    map.add_unipolar(CONTROLS, "AutoStepRate", |v| Comet(Set(AutoStepRate(v))));

    map.add_trigger(CONTROLS, "StepBackwards", Comet(Step(Direction::Backward)));
    map.add_trigger(CONTROLS, "StepForwards", Comet(Step(Direction::Forward)));

    map.add_radio_button_array(MACRO_SELECT_RADIO_BUTTON, |v| Comet(Set(SelectMacro(v))));

    map.add_bool(MUSIC, "ShutterSoundActive", |v| {
        Comet(Set(ShutterSoundActive(v)))
    });
    map.add_bool(MUSIC, "TrigSoundActive", |v| Comet(Set(TrigSoundActive(v))));

    map.add_bool(DEBUG, "Reset", |v| Comet(Set(Reset(v))));
}

pub fn handle_comet_state_change<S>(sc: StateChange, send: &mut S)
where
    S: FnMut(OscMessage),
{
    use StateChange::*;
    match sc {
        // Most controls do not have talkback due to network latency issues.
        // Consider changing this.
        SelectMacro(v) => {
            if let Err(e) = MACRO_SELECT_RADIO_BUTTON.set(v, send) {
                error!("Macro select update error: {}.", e);
            }
        }
        _ => (),
    }
}
