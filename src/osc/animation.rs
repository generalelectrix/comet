use rosc::OscMessage;
use tunnels::clock_bank::{ClockIdxExt, N_CLOCKS};

use crate::fixture::FixtureControlMessage;
use crate::osc::HandleStateChange;
use crate::osc::{send_button, send_float, ControlMap, MapControls, RadioButton};

use tunnels::animation::{ControlMessage, StateChange, Waveform::*};

const GROUP: &str = "Animation";

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
        use FixtureControlMessage::{Animation, Error as ControlError};
        use StateChange::*;
        map.add_radio_button_array(WAVEFORM_SELECT, |v| {
            match v {
                0 => Some(Sine),
                1 => Some(Triangle),
                2 => Some(Square),
                3 => Some(Sawtooth),
                _ => None,
            }
            .map(|waveform| Animation(Set(Waveform(waveform))))
            .unwrap_or_else(|| ControlError(format!("waveform select out of range: {v}")))
        });

        map.add_bipolar(GROUP, SPEED, |v| Animation(Set(Speed(v))));
        map.add_unipolar(GROUP, SIZE, |v| Animation(Set(Size(v))));
        map.add_unipolar(GROUP, DUTY_CYCLE, |v| Animation(Set(DutyCycle(v))));
        map.add_unipolar(GROUP, SMOOTHING, |v| Animation(Set(Smoothing(v))));

        map.add_radio_button_array(N_PERIODS_SELECT, |v| Animation(Set(NPeriods(v))));
        map.add_radio_button_array(CLOCK_SOURCE, |v| {
            if v == 0 {
                Animation(SetClockSource(None))
            } else {
                Animation(SetClockSource(Some(ClockIdxExt(v - 1))))
            }
        });
        map.add_trigger(GROUP, PULSE, Animation(TogglePulse));
        map.add_trigger(GROUP, INVERT, Animation(ToggleInvert));
        map.add_trigger(GROUP, STANDING, Animation(ToggleStanding));
        map.add_trigger(GROUP, USE_AUDIO_SPEED, Animation(ToggleUseAudioSpeed));
        map.add_trigger(GROUP, USE_AUDIO_SIZE, Animation(ToggleUseAudioSize));
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
