"""Abstraction layer on top of the DMX interface to a Coemar Comet."""

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
        self.dmx_addr = dmx_addr

        self.shutter_open = False
        self.strobing = False
        self.strobe_rate = 0.0
        self.shutter_sound_active = False

        self.macro_pattern = 0
        self.mirror_speed = 0.0

        self.trigger_state = TriggerState()

        self.reset = False

    def render(self, dmx_univ):
        """Render this Comet into a DMX universe."""

        # render the shutter value
        dmx_univ[self.dmx_addr] = self._render_shutter()
        dmx_univ[self.dmx_addr + 1] = self._game_dmx_vals[self.macro_pattern]
        dmx_univ[self.dmx_addr + 2] = self._render_mspeed()
        dmx_univ[self.dmx_addr + 3] = self.trigger_state.render_trigger()
        # reset
        dmx_univ[self.dmx_addr + 4] = 255 if self.reset else 0

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


def unit_float_to_range(start, end, value):
    return int((end-start)*value)+start

# clever Python enum trick
# Idle isn't a great name, as the comet could be in music trig or auto trig
# the point here is that the Idle state is any state that implies a DMX
# value outside the take a step ranges
Idle, StepForwards, StepBackwards = range(3)
Forwards, Backwards = range(2)

class TriggerState(object):
    """Helper object to contain Comet trigger state.

    This is tricky, as the trigger interface cannot be stateless.
    """

    _n_frames_for_man_step = 1
    _step_f_dmx_val = 108
    _step_b_dmx_val = 142
    _stop_dmx_val = 124
    _music_dmx_val = 50


    def __init__(self):

        # stateless quantities
        self.music_trigger = False
        self.auto_step_rate = 0.0
        self.auto_step = False

        # should the state machine take a step?
        # if so, which direction?
        self._take_a_step = False
        self._direction = None

        # is the state machine busy processing an operation?
        # if so, UI events may be dropped
        self._busy = False

        self._state = Idle

    def step_forwards(self):
        if not self._busy:
            self._take_a_step = True
            self._direction = Forwards

    def step_backwards(self):
        if not self._busy:
            self._take_a_step = True
            self._direction = Backwards

    def render_trigger(self):
        """Render the trigger state to DMX.

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
        # yikes!
        # first check to see if we need to take a step:
        last_state = self._state
        if self._take_a_step:
            targ_state = StepForwards if self._direction is Forwards else StepBackwards

            # if our last state was a different state, no problem
            if last_state != targ_state:
                self._take_a_step = False
                self._busy = False

                if self._direction is Forwards:
                    self._state = StepForwards
                    return self._step_f_dmx_val
                else:
                    self._state = StepBackwards
                    return self._step_b_dmx_val
            # otherwise, we need to take an intermediate step to the "stopped"
            # state and THEN to the step state
            self._busy = True
            self._state = Idle
            return self._stop_dmx_val
        # if we're not taking a step, easy sauce
        elif self.music_trigger:
            return self._music_dmx_val
        elif self.auto_step:
            return unit_float_to_range(151, 255, self.auto_step_rate)
        else:
            return self._stop_dmx_val

