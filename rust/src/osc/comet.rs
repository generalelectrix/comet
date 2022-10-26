use super::ControlMap;
use crate::{
    comet::{ControlMessage, StateChange, Step as Direction},
    osc::quadratic,
};

// Control group names.
const CONTROLS: &str = "Controls";
const MUSIC: &str = "Music";
const DEBUG: &str = "Debug";

pub fn map_comet_controls(map: &mut ControlMap<ControlMessage>) {
    use ControlMessage::*;
    use StateChange::*;
    map.add_bool(CONTROLS, "Shutter", |v| Set(Shutter(v)));
    map.add_bool(CONTROLS, "Strobe", |v| Set(Strobe(v)));
    map.add_unipolar(CONTROLS, "StrobeRate", |v| Set(StrobeRate(quadratic(v))));
    map.add_unipolar(CONTROLS, "Mspeed", |v| Set(MirrorSpeed(v)));
    map.add_bool(CONTROLS, "AutoStep", |v| Set(AutoStep(v)));
    map.add_unipolar(CONTROLS, "AutoStepRate", |v| Set(AutoStepRate(v)));

    map.add_trigger(CONTROLS, "StepBackwards", Step(Direction::Backward));
    map.add_trigger(CONTROLS, "StepForwards", Step(Direction::Forward));

    map.add_1d_radio_button_array(CONTROLS, "SelectMacro", |v| Set(SelectMacro(v)));

    map.add_bool(MUSIC, "ShutterSoundActive", |v| Set(ShutterSoundActive(v)));
    map.add_bool(MUSIC, "TrigSoundActive", |v| Set(TrigSoundActive(v)));

    map.add_bool(DEBUG, "Reset", |v| Set(Reset(v)));
}
