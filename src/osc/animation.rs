use rosc::OscMessage;
use tunnels::clock_bank::{ClockIdxExt, N_CLOCKS};

use crate::animation::ControlMessage::Animation as WrapAnimation;
use crate::animation::GroupSelection;
use crate::fixture::{FixtureControlMessage, N_ANIM};
use crate::osc::HandleStateChange;
use crate::osc::{send_button, send_float, ControlMap, MapControls, RadioButton};

use tunnels::animation::{ControlMessage, StateChange, Waveform::*};

use super::label_array::LabelArray;

const GROUP: &str = "Animation";

// Base animation system

// knobs
const SPEED: &str = "Speed";
const SIZE: &str = "Size";
const DUTY_CYCLE: &str = "DutyCycle";
const SMOOTHING: &str = "Smoothing";

// assorted parameters
const PULSE: &str = "Pulse";
const INVERT: &str = "Invert";
const USE_AUDIO_SIZE: &str = "UseAudioSize";
const USE_AUDIO_SPEED: &str = "UseAudioSpeed";
const STANDING: &str = "Standing";

const WAVEFORM_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Waveform",
    n: 4,
    x_primary_coordinate: false,
};

const N_PERIODS_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "NPeriods",
    n: 16,
    x_primary_coordinate: false,
};

const CLOCK_SOURCE: RadioButton = RadioButton {
    group: GROUP,
    control: "ClockSource",
    n: N_CLOCKS + 1,
    x_primary_coordinate: false,
};

pub struct AnimationControls;

impl MapControls for AnimationControls {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use ControlMessage::*;
        use FixtureControlMessage::{Animation as FixtureAnimation, Error as ControlError};
        use StateChange::*;
        map.add_radio_button_array(WAVEFORM_SELECT, |v| {
            match v {
                0 => Some(Sine),
                1 => Some(Triangle),
                2 => Some(Square),
                3 => Some(Sawtooth),
                _ => None,
            }
            .map(|waveform| FixtureAnimation(WrapAnimation(Set(Waveform(waveform)))))
            .unwrap_or_else(|| ControlError(format!("waveform select out of range: {v}")))
        });

        map.add_bipolar(GROUP, SPEED, |v| {
            FixtureAnimation(WrapAnimation(Set(Speed(v))))
        });
        map.add_unipolar(GROUP, SIZE, |v| {
            FixtureAnimation(WrapAnimation(Set(Size(v))))
        });
        map.add_unipolar(GROUP, DUTY_CYCLE, |v| {
            FixtureAnimation(WrapAnimation(Set(DutyCycle(v))))
        });
        map.add_unipolar(GROUP, SMOOTHING, |v| {
            FixtureAnimation(WrapAnimation(Set(Smoothing(v))))
        });

        map.add_radio_button_array(N_PERIODS_SELECT, |v| {
            FixtureAnimation(WrapAnimation(Set(NPeriods(v))))
        });
        map.add_radio_button_array(CLOCK_SOURCE, |v| {
            if v == 0 {
                FixtureAnimation(WrapAnimation(SetClockSource(None)))
            } else {
                FixtureAnimation(WrapAnimation(SetClockSource(Some(ClockIdxExt(v - 1)))))
            }
        });
        map.add_trigger(GROUP, PULSE, FixtureAnimation(WrapAnimation(TogglePulse)));
        map.add_trigger(GROUP, INVERT, FixtureAnimation(WrapAnimation(ToggleInvert)));
        map.add_trigger(
            GROUP,
            STANDING,
            FixtureAnimation(WrapAnimation(ToggleStanding)),
        );
        map.add_trigger(
            GROUP,
            USE_AUDIO_SPEED,
            FixtureAnimation(WrapAnimation(ToggleUseAudioSpeed)),
        );
        map.add_trigger(
            GROUP,
            USE_AUDIO_SIZE,
            FixtureAnimation(WrapAnimation(ToggleUseAudioSize)),
        );

        TargetAndSelectControls.map_controls(map);
    }
}

// Targeting/selection

const N_ANIM_TARGET: usize = 11;
const N_ANIM_GROUP: usize = 8;

const ANIMATION_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Select",
    n: N_ANIM,
    x_primary_coordinate: false,
};

const ANIMATION_TARGET_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Target",
    n: N_ANIM_TARGET,
    x_primary_coordinate: false,
};

const ANIMATION_TARGET_LABELS: LabelArray = LabelArray {
    group: GROUP,
    control: "TargetLabel",
    n: N_ANIM_TARGET,
    empty_label: "XXXXXX",
};

const ANIMATION_GROUP_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Group",
    n: N_ANIM_GROUP,
    x_primary_coordinate: false,
};

const ANIMATION_GROUP_LABELS: LabelArray = LabelArray {
    group: GROUP,
    control: "GroupLabel",
    n: N_ANIM_GROUP,
    empty_label: "XXXXXX",
};

struct TargetAndSelectControls;

impl MapControls for TargetAndSelectControls {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use crate::animation::ControlMessage;
        use FixtureControlMessage::Animation;

        map.add_radio_button_array(ANIMATION_GROUP_SELECT, |msg| {
            Animation(ControlMessage::SelectGroup(msg))
        });
        map.add_radio_button_array(ANIMATION_TARGET_SELECT, |msg| {
            Animation(ControlMessage::Target(msg))
        });
        map.add_radio_button_array(ANIMATION_SELECT, |msg| {
            Animation(ControlMessage::SelectAnimation(msg))
        });
    }
}

impl HandleStateChange<crate::animation::StateChange> for AnimationControls {
    fn emit_state_change<S>(sc: crate::animation::StateChange, send: &mut S)
    where
        S: FnMut(OscMessage),
    {
        match sc {
            crate::animation::StateChange::Animation(msg) => {
                AnimationControls::emit_state_change(msg, send)
            }
            crate::animation::StateChange::SelectAnimation(msg) => ANIMATION_SELECT.set(msg, send),
            crate::animation::StateChange::SelectGroup(msg) => {
                ANIMATION_GROUP_SELECT.set(msg.0, send)
            }
            crate::animation::StateChange::Target(msg) => ANIMATION_TARGET_SELECT.set(msg, send),
            crate::animation::StateChange::TargetLabels(labels) => {
                ANIMATION_TARGET_LABELS.set(labels.into_iter(), send)
            }
            crate::animation::StateChange::GroupLabels(labels) => {
                ANIMATION_GROUP_LABELS.set(labels.into_iter(), send)
            }
        }
    }
}

impl HandleStateChange<StateChange> for AnimationControls {
    fn emit_state_change<S>(sc: StateChange, send: &mut S)
    where
        S: FnMut(OscMessage),
    {
        use StateChange::*;
        match sc {
            Waveform(v) => WAVEFORM_SELECT.set(
                match v {
                    Sine => 0,
                    Triangle => 1,
                    Square => 2,
                    Sawtooth => 3,
                },
                send,
            ),
            Speed(v) => send_float(GROUP, SPEED, v, send),
            Size(v) => send_float(GROUP, SIZE, v, send),
            DutyCycle(v) => send_float(GROUP, DUTY_CYCLE, v, send),
            Smoothing(v) => send_float(GROUP, SMOOTHING, v, send),

            NPeriods(v) => N_PERIODS_SELECT.set(v, send),
            ClockSource(maybe_clock) => {
                CLOCK_SOURCE.set(maybe_clock.map(|v| usize::from(v) + 1).unwrap_or(0), send)
            }

            Pulse(v) => send_button(GROUP, PULSE, v, send),
            Standing(v) => send_button(GROUP, STANDING, v, send),
            Invert(v) => send_button(GROUP, INVERT, v, send),
            UseAudioSize(v) => send_button(GROUP, USE_AUDIO_SIZE, v, send),
            UseAudioSpeed(v) => send_button(GROUP, USE_AUDIO_SPEED, v, send),
        }
    }
}
