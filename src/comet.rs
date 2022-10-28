use log::{debug, error};
use number::UnipolarFloat;
use std::{collections::VecDeque, time::Duration};

use crate::dmx::DmxAddr;
use crate::fixture::{EmitStateChange as EmitShowStateChange, StateChange as ShowStateChange};
use crate::util::unipolar_to_range;

pub struct Comet {
    dmx_index: usize,
    shutter_open: bool,
    strobing: bool,
    strobe_rate: UnipolarFloat,
    shutter_sound_active: bool,
    macro_pattern: usize,
    mirror_speed: UnipolarFloat,
    trigger_state: TriggerState,
    reset: bool,
}

impl Comet {
    const GAME_DMX_VALS: [u8; 10] = [12, 35, 65, 85, 112, 140, 165, 190, 212, 240];

    pub fn new(dmx_addr: DmxAddr) -> Self {
        Self {
            dmx_index: dmx_addr - 1,
            shutter_open: false,
            strobing: false,
            strobe_rate: UnipolarFloat::ZERO,
            shutter_sound_active: false,
            macro_pattern: 0,
            mirror_speed: UnipolarFloat::ZERO,
            trigger_state: TriggerState::new(),
            reset: false,
        }
    }

    pub fn update(&mut self, delta_t: Duration) {
        self.trigger_state.update(delta_t);
    }

    /// Render into the provided DMX universe.
    pub fn render(&self, dmx_univ: &mut [u8]) {
        dmx_univ[self.dmx_index] = self.render_shutter();
        dmx_univ[self.dmx_index + 1] = Self::GAME_DMX_VALS[self.macro_pattern];
        dmx_univ[self.dmx_index + 2] = self.render_mspeed();
        dmx_univ[self.dmx_index + 3] = self.trigger_state.render();
        dmx_univ[self.dmx_index + 4] = if self.reset { 255 } else { 0 };
        debug!("{:?}", &dmx_univ[self.dmx_index..self.dmx_index + 5]);
    }

    fn render_shutter(&self) -> u8 {
        if !self.shutter_open {
            0
        } else if self.shutter_sound_active {
            125
        } else if self.strobing {
            unipolar_to_range(151, 255, self.strobe_rate)
        } else {
            75
        }
    }

    fn render_mspeed(&self) -> u8 {
        unipolar_to_range(0, 255, self.mirror_speed)
    }

    /// Emit the current value of all controllable state.
    pub fn emit_state<E: EmitStateChange>(&self, emitter: &mut E) {
        use StateChange::*;
        emitter.emit(Shutter(self.shutter_open));
        emitter.emit(Strobe(self.strobing));
        emitter.emit(StrobeRate(self.strobe_rate));
        emitter.emit(ShutterSoundActive(self.shutter_sound_active));
        emitter.emit(SelectMacro(self.macro_pattern));
        emitter.emit(MirrorSpeed(self.mirror_speed));
        emitter.emit(TrigSoundActive(self.trigger_state.music_trigger));
        emitter.emit(AutoStep(self.trigger_state.auto_step));
        emitter.emit(AutoStepRate(self.trigger_state.auto_step_rate));
        emitter.emit(Reset(self.reset));
    }

    pub fn control<E: EmitStateChange>(&mut self, msg: ControlMessage, emitter: &mut E) {
        use ControlMessage::*;
        match msg {
            Set(sc) => self.handle_state_change(sc, emitter),
            Step(direction) => self.trigger_state.enqueue_step(direction),
        }
    }

    fn handle_state_change<E: EmitStateChange>(&mut self, sc: StateChange, emitter: &mut E) {
        use StateChange::*;
        match sc {
            Shutter(v) => self.shutter_open = v,
            Strobe(v) => self.strobing = v,
            StrobeRate(v) => self.strobe_rate = v,
            ShutterSoundActive(v) => self.shutter_sound_active = v,
            SelectMacro(v) => {
                if v >= Self::GAME_DMX_VALS.len() {
                    error!("Macro index {} out of range.", v);
                    return;
                }
                self.macro_pattern = v;
            }
            MirrorSpeed(v) => self.mirror_speed = v,
            TrigSoundActive(v) => self.trigger_state.music_trigger = v,
            AutoStep(v) => self.trigger_state.auto_step = v,
            AutoStepRate(v) => self.trigger_state.auto_step_rate = v,
            Reset(v) => self.reset = v,
        };
        emitter.emit(sc);
    }
}

/// Manage Comet trigger state.
struct TriggerState {
    music_trigger: bool,
    auto_step_rate: UnipolarFloat,
    auto_step: bool,
    /// queue of step actions to process
    steps_to_take: std::collections::VecDeque<Step>,
    /// what state was this machine in on the last update?
    prior_state: Stepping,
    /// how many frames should we hold current output before re-updating?
    updates_to_hold: usize,
    current_output_value: u8,
}

impl TriggerState {
    const DMX_VAL_STEP_FORWARD: u8 = 108;
    const DMX_VAL_STEP_BACKWARD: u8 = 142;
    const DMX_VAL_STOP: u8 = 124;
    const DMX_VAL_MUSIC_TRIG: u8 = 50;

    /// rendering to the enttec is asynchronous and frame tearing is a problem
    /// how many updates should we hold the current output before processing another?
    const UPDATES_TO_HOLD: usize = 3;

    fn new() -> Self {
        Self {
            music_trigger: false,
            auto_step_rate: UnipolarFloat::ZERO,
            auto_step: false,
            steps_to_take: VecDeque::new(),
            prior_state: Stepping::Idle,
            updates_to_hold: 0,
            current_output_value: Self::DMX_VAL_STOP,
        }
    }

    fn enqueue_step(&mut self, direction: Step) {
        self.steps_to_take.push_back(direction);
    }

    fn render(&self) -> u8 {
        self.current_output_value
    }

    pub fn update(&mut self, _: Duration) {
        if self.updates_to_hold > 0 {
            self.updates_to_hold -= 1;
        } else {
            self.current_output_value = self._update();
        }
    }

    /// Update the DMX trigger state value.
    ///
    /// This is a fairly complex action, as the step interface at the DMX level
    /// is kinda hokey.
    ///
    /// I give top priority to the mechanism by which we achieve manual stepping.
    /// After that, music responsive mode.  After that, automatic stepping.
    /// The trigger UI should make it clear that music trigger and auto trigger
    /// are mutually exclusive options.
    fn _update(&mut self) -> u8 {
        // what needs to happen to take a step:
        // the dmx value needs to go from its current state to the step value
        // if the current value is the step value, we need to leave and come back again
        if let Some(next_step) = self.steps_to_take.pop_front() {
            // hold this output for a minimum frame count before processing next
            self.updates_to_hold += Self::UPDATES_TO_HOLD - 1;

            if next_step == Step::Forward && self.prior_state != Stepping::Forwards {
                // can take this step, transition to forward
                self.prior_state = Stepping::Forwards;
                return Self::DMX_VAL_STEP_FORWARD;
            } else if next_step == Step::Backward && self.prior_state != Stepping::Backwards {
                // can take this step, transition to backward
                self.prior_state = Stepping::Backwards;
                return Self::DMX_VAL_STEP_BACKWARD;
            } else {
                // we're in the same state as the step we need to take
                // transition to Idle, then take the step on the next update
                self.steps_to_take.push_front(next_step);
                self.prior_state = Stepping::Idle;
                return Self::DMX_VAL_STOP;
            }
        }

        // nothing in the step queue so the state machine is idle
        self.prior_state = Stepping::Idle;

        // if we're not taking a step, easy sauce
        if self.music_trigger {
            return Self::DMX_VAL_MUSIC_TRIG;
        } else if self.auto_step {
            return unipolar_to_range(151, 255, self.auto_step_rate);
        } else {
            return Self::DMX_VAL_STOP;
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum Stepping {
    Idle,
    Forwards,
    Backwards,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Step {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug)]
pub enum ControlMessage {
    Set(StateChange),
    Step(Step),
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    Shutter(bool),
    Strobe(bool),
    StrobeRate(UnipolarFloat),
    ShutterSoundActive(bool),
    SelectMacro(usize),
    MirrorSpeed(UnipolarFloat),
    TrigSoundActive(bool),
    AutoStep(bool),
    AutoStepRate(UnipolarFloat),
    Reset(bool),
}

pub trait EmitStateChange {
    fn emit(&mut self, sc: StateChange);
}

impl<T: EmitShowStateChange> EmitStateChange for T {
    fn emit(&mut self, sc: StateChange) {
        self.emit(ShowStateChange::Comet(sc));
    }
}
