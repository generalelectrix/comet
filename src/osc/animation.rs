use anyhow::anyhow;
use tunnels::clock_bank::{ClockIdxExt, N_CLOCKS};

use crate::animation::AnimationUIState;
use crate::animation::ControlMessage as AnimationControlMessage;
use crate::animation::ControlMessage::Animation as WrapAnimation;

use crate::fixture::animation_target::N_ANIM;
use crate::fixture::ControlMessagePayload;
use crate::osc::HandleOscStateChange;
use crate::osc::{send_float, MapControls, RadioButton};

use tunnels::animation::{ControlMessage, StateChange, Waveform::*};

use super::basic_controls::{button, Button};
use super::label_array::LabelArray;
use super::GroupControlMap;

pub(crate) const GROUP: &str = "Animation";

// Base animation system

// knobs
const SPEED: &str = "Speed";
const SIZE: &str = "Size";
const DUTY_CYCLE: &str = "DutyCycle";
const SMOOTHING: &str = "Smoothing";

// assorted parameters
const PULSE: Button = button(GROUP, "Pulse");
const INVERT: Button = button(GROUP, "Invert");
const USE_AUDIO_SIZE: Button = button(GROUP, "UseAudioSize");
const USE_AUDIO_SPEED: Button = button(GROUP, "UseAudioSpeed");
const STANDING: Button = button(GROUP, "Standing");

const WAVEFORM_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Waveform",
    n: 5,
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

impl MapControls for AnimationUIState {
    fn group(&self) -> &'static str {
        GROUP
    }

    fn map_controls(&self, map: &mut GroupControlMap<ControlMessagePayload>) {
        use ControlMessage::*;
        use ControlMessagePayload::Animation as FixtureAnimation;
        use StateChange::*;
        WAVEFORM_SELECT.map_fallible(map, |v| {
            match v {
                0 => Some(Sine),
                1 => Some(Triangle),
                2 => Some(Square),
                3 => Some(Sawtooth),
                4 => Some(Constant),
                _ => None,
            }
            .map(|waveform| FixtureAnimation(WrapAnimation(Set(Waveform(waveform)))))
            .ok_or_else(|| anyhow!("waveform select out of range: {v}"))
        });

        map.add_bipolar(SPEED, |v| FixtureAnimation(WrapAnimation(Set(Speed(v)))));
        map.add_unipolar(SIZE, |v| FixtureAnimation(WrapAnimation(Set(Size(v)))));
        map.add_unipolar(DUTY_CYCLE, |v| {
            FixtureAnimation(WrapAnimation(Set(DutyCycle(v))))
        });
        map.add_unipolar(SMOOTHING, |v| {
            FixtureAnimation(WrapAnimation(Set(Smoothing(v))))
        });

        N_PERIODS_SELECT.map(map, |v| FixtureAnimation(WrapAnimation(Set(NPeriods(v)))));
        CLOCK_SOURCE.map(map, |v| {
            if v == 0 {
                FixtureAnimation(WrapAnimation(SetClockSource(None)))
            } else {
                FixtureAnimation(WrapAnimation(SetClockSource(Some(ClockIdxExt(v - 1)))))
            }
        });
        PULSE.map_trigger(map, || FixtureAnimation(WrapAnimation(TogglePulse)));
        INVERT.map_trigger(map, || FixtureAnimation(WrapAnimation(ToggleInvert)));
        STANDING.map_trigger(map, || FixtureAnimation(WrapAnimation(ToggleStanding)));
        USE_AUDIO_SPEED.map_trigger(map, || FixtureAnimation(WrapAnimation(ToggleUseAudioSpeed)));
        USE_AUDIO_SIZE.map_trigger(map, || FixtureAnimation(WrapAnimation(ToggleUseAudioSize)));

        ANIMATION_TARGET_SELECT.map(map, |msg| {
            ControlMessagePayload::Animation(AnimationControlMessage::Target(msg))
        });
        ANIMATION_SELECT.map(map, |msg| {
            ControlMessagePayload::Animation(AnimationControlMessage::SelectAnimation(msg))
        });
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        Default::default()
    }
}

// Targeting/selection

const N_ANIM_TARGET: usize = 8;

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
    empty_label: "",
};

impl HandleOscStateChange<crate::animation::StateChange> for AnimationUIState {
    fn emit_osc_state_change<S>(sc: crate::animation::StateChange, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        match sc {
            crate::animation::StateChange::Animation(msg) => Self::emit_osc_state_change(msg, send),
            crate::animation::StateChange::SelectAnimation(msg) => ANIMATION_SELECT.set(msg, send),
            crate::animation::StateChange::Target(msg) => ANIMATION_TARGET_SELECT.set(msg, send),
            crate::animation::StateChange::TargetLabels(labels) => {
                ANIMATION_TARGET_LABELS.set(labels.into_iter(), send)
            }
        }
    }
}

impl HandleOscStateChange<StateChange> for AnimationUIState {
    fn emit_osc_state_change<S>(sc: StateChange, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        use StateChange::*;
        match sc {
            Waveform(v) => WAVEFORM_SELECT.set(
                match v {
                    Sine => 0,
                    Triangle => 1,
                    Square => 2,
                    Sawtooth => 3,
                    Constant => 4,
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

            Pulse(v) => PULSE.send(v, send),
            Standing(v) => STANDING.send(v, send),
            Invert(v) => INVERT.send(v, send),
            UseAudioSize(v) => USE_AUDIO_SIZE.send(v, send),
            UseAudioSpeed(v) => USE_AUDIO_SPEED.send(v, send),
        }
    }
}
