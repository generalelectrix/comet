use crate::fixture::generic::GenericStrobeStateChange;

use crate::fixture::PatchFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{GroupControlMap, HandleOscStateChange,  RadioButton};
use crate::{
    fixture::comet::{Comet, ControlMessage, StateChange, Step as Direction},
    osc::quadratic,
};

// Control group names.
const GROUP: &str = Comet::NAME.0;

// Buttons.
const SHUTTER: Button = button(GROUP, "Shutter");
const STROBE_ON: Button = button(GROUP, "StrobeOn");
const AUTO_STEP: Button = button(GROUP, "AutoStep");
const STEP_BACKWARDS: Button = button(GROUP, "StepBackwards");
const STEP_FORWARDS: Button = button(GROUP, "StepForwards");
const SHUTTER_SOUND_ACTIVE: Button = button(GROUP, "ShutterSoundActive");
const TRIG_SOUND_ACTIVE: Button = button(GROUP, "TrigSoundActive");
const RESET: Button = button(GROUP, "Reset");

const MACRO_SELECT_RADIO_BUTTON: RadioButton = RadioButton {
    group: GROUP,
    control: "SelectMacro",
    n: 10,
    x_primary_coordinate: true,
};

impl Comet {
    fn group(&self) -> &'static str {
        GROUP
    }

    fn map_controls(&self, map: &mut GroupControlMap<ControlMessagePayload>) {
        use ControlMessage::*;
        use StateChange::*;
        SHUTTER.map_state(map, |v| ControlMessagePayload::fixture(Set(Shutter(v))));
        STROBE_ON.map_state(map, |v| {
            ControlMessagePayload::fixture(Set(Strobe(GenericStrobeStateChange::On(v))))
        });
        map.add_unipolar("StrobeRate", |v| {
            ControlMessagePayload::fixture(Set(Strobe(GenericStrobeStateChange::Rate(quadratic(
                v,
            )))))
        });
        map.add_unipolar("Mspeed", |v| {
            ControlMessagePayload::fixture(Set(MirrorSpeed(v)))
        });
        AUTO_STEP.map_state(map, |v| ControlMessagePayload::fixture(Set(AutoStep(v))));
        map.add_unipolar("AutoStepRate", |v| {
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
