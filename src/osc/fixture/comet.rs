use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{ControlMap, HandleOscStateChange, MapControls, RadioButton};
use crate::{
    fixture::comet::{Comet, ControlMessage, StateChange, Step as Direction},
    osc::quadratic,
};

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
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        use ControlMessage::*;
        use StateChange::*;
        SHUTTER.map_state(map, |v| ControlMessagePayload::fixture(Set(Shutter(v))));
        STROBE_ON.map_state(map, |v| {
            ControlMessagePayload::fixture(Set(Strobe(GenericStrobeStateChange::On(v))))
        });
        map.add_unipolar(CONTROLS, "StrobeRate", |v| {
            ControlMessagePayload::fixture(Set(Strobe(GenericStrobeStateChange::Rate(quadratic(
                v,
            )))))
        });
        map.add_unipolar(CONTROLS, "Mspeed", |v| {
            ControlMessagePayload::fixture(Set(MirrorSpeed(v)))
        });
        AUTO_STEP.map_state(map, |v| ControlMessagePayload::fixture(Set(AutoStep(v))));
        map.add_unipolar(CONTROLS, "AutoStepRate", |v| {
            ControlMessagePayload::fixture(Set(AutoStepRate(v)))
        });

        STEP_BACKWARDS.map_trigger(map, || {
            ControlMessagePayload::fixture(Step(Direction::Backward))
        });
        STEP_FORWARDS.map_trigger(map, || {
            ControlMessagePayload::fixture(Step(Direction::Forward))
        });

        MACRO_SELECT_RADIO_BUTTON.map(map, |v| ControlMessagePayload::fixture(Set(SelectMacro(v))));

        SHUTTER_SOUND_ACTIVE.map_state(map, |v| {
            ControlMessagePayload::fixture(Set(ShutterSoundActive(v)))
        });
        TRIG_SOUND_ACTIVE.map_state(map, |v| {
            ControlMessagePayload::fixture(Set(TrigSoundActive(v)))
        });

        RESET.map_state(map, |v| ControlMessagePayload::fixture(Set(Reset(v))));
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        vec![
            (CONTROLS.to_string(), Self::NAME),
            (MUSIC.to_string(), Self::NAME),
            (DEBUG.to_string(), Self::NAME),
        ]
    }
}

impl HandleOscStateChange<StateChange> for Comet {
    fn emit_osc_state_change<S>(sc: StateChange, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
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
