use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::FixtureControlMessage;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{ControlMap, HandleStateChange, MapControls, RadioButton};
use crate::{
    fixture::comet::{Comet, ControlMessage, StateChange, Step as Direction},
    osc::quadratic,
};

use rosc::OscMessage;
// Control group names.
const CONTROLS: &str = "Controls";
const MUSIC: &str = "Music";
const DEBUG: &str = "Debug";

// Buttons.
const SHUTTER: Button = button(CONTROLS, "Shutter");
const STROBE_ON: Button = button(CONTROLS, "StrobeOn");
const AUTO_STEP: Button = button(CONTROLS, "AutoStep");
const STEP_BACKWARDS: Button = button(CONTROLS, "StepBackwards");
const STEP_FORWARDS: Button = button(CONTROLS, "StepForwards");
const SHUTTER_SOUND_ACTIVE: Button = button(MUSIC, "ShutterSoundActive");
const TRIG_SOUND_ACTIVE: Button = button(MUSIC, "TrigSoundActive");
const RESET: Button = button(DEBUG, "Reset");

const MACRO_SELECT_RADIO_BUTTON: RadioButton = RadioButton {
    group: CONTROLS,
    control: "SelectMacro",
    n: 10,
    x_primary_coordinate: true,
};

impl MapControls for Comet {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use ControlMessage::*;
        use FixtureControlMessage::Comet;
        use StateChange::*;
        SHUTTER.map_state(map, |v| Comet(Set(Shutter(v))));
        STROBE_ON.map_state(map, |v| Comet(Set(Strobe(GenericStrobeStateChange::On(v)))));
        map.add_unipolar(CONTROLS, "StrobeRate", |v| {
            Comet(Set(Strobe(GenericStrobeStateChange::Rate(quadratic(v)))))
        });
        map.add_unipolar(CONTROLS, "Mspeed", |v| Comet(Set(MirrorSpeed(v))));
        AUTO_STEP.map_state(map, |v| Comet(Set(AutoStep(v))));
        map.add_unipolar(CONTROLS, "AutoStepRate", |v| Comet(Set(AutoStepRate(v))));

        STEP_BACKWARDS.map_trigger(map, Comet(Step(Direction::Backward)));
        STEP_FORWARDS.map_trigger(map, Comet(Step(Direction::Forward)));

        MACRO_SELECT_RADIO_BUTTON.map(map, |v| Comet(Set(SelectMacro(v))));

        SHUTTER_SOUND_ACTIVE.map_state(map, |v| Comet(Set(ShutterSoundActive(v))));
        TRIG_SOUND_ACTIVE.map_state(map, |v| Comet(Set(TrigSoundActive(v))));

        RESET.map_state(map, |v| Comet(Set(Reset(v))));
    }
}

impl HandleStateChange<StateChange> for Comet {
    fn emit_state_change<S>(sc: StateChange, send: &mut S, talkback: crate::osc::TalkbackMode)
    where
        S: FnMut(OscMessage),
    {
        use StateChange::*;
        #[allow(clippy::single_match)]
        match sc {
            // Most controls do not have talkback due to network latency issues.
            // Consider changing this.
            SelectMacro(v) => MACRO_SELECT_RADIO_BUTTON.set(v, send),
            _ => (),
        }
    }
}
