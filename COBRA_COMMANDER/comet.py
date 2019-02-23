"""Abstraction layer on top of the DMX interface to a Coemar Comet."""
from collections import deque
from utils import unit_float_to_range, ignore_all_but_1, quadratic_fader
import logging as log

class Comet(object):
    """Better control abstraction for the Coemar Comet.

    Shutter control precedence (highest to lowest):
        shutter_open?
        shutter_sound_active?
        strobing? (with rate)

    Trigger control precendence:
        music_trigger?
        auto_step?
        manual step trigger action?
    """

    _game_dmx_vals = [12, 35, 65, 85, 112, 140, 165, 190, 212, 240]

    def __init__(self, dmx_addr):
        """Create a new wrapper for a comet."""
        self.dmx_addr = dmx_addr - 1

        self.shutter_open = False
        self.strobing = False
        self.strobe_rate = 0.0
        self.shutter_sound_active = False

        self.macro_pattern = 0
        self.mirror_speed = 0.0

        self.trigger_state = TriggerState()

        self.reset = False

    def update(self, timestep):
        self.trigger_state.update(timestep)

    def render(self, dmx_univ):
        """Render this Comet into a DMX universe."""

        # render the shutter value
        dmx_univ[self.dmx_addr] = self._render_shutter()
        dmx_univ[self.dmx_addr + 1] = self._game_dmx_vals[self.macro_pattern]
        dmx_univ[self.dmx_addr + 2] = self._render_mspeed()
        dmx_univ[self.dmx_addr + 3] = self.trigger_state.render()
        # reset
        dmx_univ[self.dmx_addr + 4] = 255 if self.reset else 0

        log.debug(dmx_univ[self.dmx_addr:self.dmx_addr+5])

    def _render_shutter(self):
        """Render the shutter state into DMX."""
        if not self.shutter_open:
            return 0
        elif self.shutter_sound_active:
            return 125
        elif self.strobing:
            return unit_float_to_range(151, 255, self.strobe_rate)
        else:
            return 75

    def _render_mspeed(self):
        return unit_float_to_range(0, 255, self.mirror_speed)

# Idle isn't a great name, as the comet could be in music trig or auto trig
# the point here is that the Idle state is any state that implies a DMX
# value outside the take a step ranges
Idle, SteppingForwards, SteppingBackwards = "I", "SF", "SB"
Forward, Backward = "F", "B"

class TriggerState(object):
    """Helper object to contain Comet trigger state."""
    # rendering to the enttec is asynchronous and frame tearing is a problem
    # how many updates should we hold the current output before processing another?
    _updates_to_hold = 3

    _step_forward_dmx_val = 108
    _step_backward_dmx_val = 142
    _stop_dmx_val = 124
    _music_dmx_val = 50

    def __init__(self):

        # stateless quantities
        self.music_trigger = False
        self.auto_step_rate = 0.0
        self.auto_step = False

        # queue of step actions to process
        self.steps_to_take = deque()

        # what state was this machine in on the last update?
        self.prior_state = Idle

        # how many frames should we hold current output before re-updating?
        self.updates_to_hold = 0

        self.current_output_value = self._stop_dmx_val

    def step_forwards(self):
        self.steps_to_take.appendleft(Forward)

    def step_backwards(self):
        self.steps_to_take.appendleft(Backward)

    def render(self):
        return self.current_output_value

    def update(self, _):
        if self.updates_to_hold > 0:
            self.updates_to_hold -= 1
        else:
            self.current_output_value = self._update()

    def _update(self):
        """Update the DMX trigger state value.

        This is a fairly complex action, as the step interface at the DMX level
        is kinda hokey.

        I give top priority to the mechanism by which we achieve manual stepping.
        After that, music responsive mode.  After that, automatic stepping.
        The trigger UI should make it clear that music trigger and auto trigger
        are mutually exclusive options.
        """
        # what needs to happen to take a step:
        # the dmx value needs to go from its current state to the step value
        # if the current value is the step value, we need to leave and come back again
        if self.steps_to_take:

            next_step = self.steps_to_take[-1]
            # hold this output for a minimum frame count before processing next
            self.updates_to_hold += self._updates_to_hold - 1

            if next_step == Forward and self.prior_state != SteppingForwards:
                # can take this step, transition to forward
                self.prior_state = SteppingForwards
                self.steps_to_take.pop()
                return self._step_forward_dmx_val
            elif next_step == Backward and self.prior_state != SteppingBackwards:
                # can take this step, transition to backward
                self.prior_state = SteppingBackwards
                self.steps_to_take.pop()
                return self._step_backward_dmx_val

            else:
                # we're in the same state as the step we need to take
                # transition to Idle, then take the step on the next update
                self.prior_state = Idle
                return self._stop_dmx_val

        # nothing in the step queue so the state machine is idle
        self.prior_state = Idle

        # if we're not taking a step, easy sauce
        if self.music_trigger:
            return self._music_dmx_val
        elif self.auto_step:
            return unit_float_to_range(151, 255, self.auto_step_rate)
        else:
            return self._stop_dmx_val

# controls and control actions

(Shutter,
 Strobe,
 StrobeRate,
 ShutterSoundActive,
 SelectMacro,
 Mspeed,
 Reset,
 StepForwards,
 StepBackwards,
 TrigSoundActive,
 AutoStep,
 AutoStepRate) = range(12)

# control actions
def shutter_state(comet, state):
    """bool"""
    comet.shutter_open = state

def strobe_state(comet, state):
    """bool"""
    comet.strobing = state

def strobe_rate(comet, rate):
    """float in [0,1]"""
    comet.strobe_rate = rate

def shutter_sound_active(comet, state):
    """bool"""
    comet.shutter_sound_active = state

def select_macro_pattern(comet, pattern):
    """int in [0,9]"""
    comet.macro_pattern = pattern

def mspeed(comet, mspeed):
    """float in [0,1]"""
    comet.mirror_speed = mspeed

def reset(comet, reset):
    """bool"""
    comet.reset = reset

def step_forwards(comet, _):
    comet.trigger_state.step_forwards()

def step_backwards(comet, _):
    comet.trigger_state.step_backwards()

def trigger_sound_active(comet, state):
    """bool"""
    comet.trigger_state.music_trigger = state

def auto_step(comet, state):
    """bool"""
    comet.trigger_state.auto_step = state

def auto_step_rate(comet, rate):
    """float in [0,1]"""
    comet.trigger_state.auto_step_rate = rate


# control mapping
control_map = {
    Shutter: shutter_state,
    Strobe: strobe_state,
    StrobeRate: strobe_rate,
    ShutterSoundActive: shutter_sound_active,
    SelectMacro: select_macro_pattern,
    Mspeed: mspeed,
    Reset: reset,
    StepForwards: step_forwards,
    StepBackwards: step_backwards,
    TrigSoundActive: trigger_sound_active,
    AutoStep: auto_step,
    AutoStepRate: auto_step_rate,}


def setup_controls(cont):

    # make groups
    cont.create_control_group('Controls')
    cont.create_control_group('Music')
    cont.create_control_group('Debug')

    # add controls
    cont.create_simple_control('Controls', 'Shutter', Shutter)
    cont.create_simple_control('Controls', 'Strobe', Strobe)
    cont.create_simple_control('Controls', 'StrobeRate', StrobeRate, quadratic_fader)
    cont.create_simple_control('Controls', 'Mspeed', Mspeed)
    cont.create_simple_control('Controls', 'AutoStep', AutoStep)
    cont.create_simple_control('Controls', 'AutoStepRate', AutoStepRate)

    cont.create_simple_control('Controls', 'StepBackwards', StepBackwards, ignore_all_but_1)
    cont.create_simple_control('Controls', 'StepForwards', StepForwards, ignore_all_but_1)

    cont.create_radio_button_control('Controls', 'SelectMacro', (10,1), SelectMacro)

    cont.create_simple_control('Music', 'ShutterSoundActive', ShutterSoundActive)
    cont.create_simple_control('Music', 'TrigSoundActive', TrigSoundActive)

    cont.create_simple_control('Debug', 'Reset', Reset)

