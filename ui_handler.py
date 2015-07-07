"""OSC and MIDI-based UI for comet control."""

class EventHandler(object):
    """Object to manage an incoming control stream."""
    def __init__(self):
        self.handlers = {}

    def register_handler(self, channel, handler):
        self.handlers[channel] = handler

    def unregister_handler(self, channel):
        try:
            del self.handlers[channel]
        except KeyError:
            pass

    def process_event(self, channel, event):
        self.handlers[channel](event)



class CometUI(object):
    """UI-facing API to an underlying Comet object."""
    def __init__(self, comet):
        self.comet = comet

    # control actions
    def shutter_state(self, state):
        """bool"""
        self.comet.shutter_open = state

    def strobe_state(self, state):
        """bool"""
        self.comet.strobing = state

    def strobe_rate(self, rate):
        """float in [0,1]"""
        self.comet.strobe_rate = rate

    def shutter_sound_active(self, state):
        """bool"""
        self.comet.shutter_sound_active = state

    def select_macro_pattern(self, pattern):
        """int in [0,9]"""
        self.comet.macro_pattern = pattern

    def mspeed(self, mspeed):
        """float in [0,1]"""
        self.comet.mirror_speed = mspeed

    def reset(self, reset):
        """bool"""
        self.comet.reset = reset

    def step_forwards(self):
        self.comet.trigger_state.step_forwards()

    def step_backwards(self):
        self.comet.trigger_state.step_forwards()

    def music_trigger(self, state):
        """bool"""
        self.comet.trigger_state.music_trigger = state

    def auto_trigger(self, state):
        """bool"""
        self.comet.trigger_state.auto_step = state

    def auto_trigger_rate(self, rate):
        """float in [0,1]"""
        self.comet.trigger_state.auto_step_rate = rate