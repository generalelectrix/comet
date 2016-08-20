
from utils import unit_float_to_range

def _render_strobe(state, intensity, rate):
    if state:
        return unit_float_to_range(0, 255, intensity), unit_float_to_range(0, 255, rate)
    else:
        return (0, 0)

class Lumasphere (object):
    """Control abstraction for the lumapshere.

    lumasphere DMX profile:

    1: outer ball rotation speed
    note: requires a value of ~17% in order to be activated
    (ball start button)

    2: outer ball rotation direction
    split halfway

    3: color wheel rotation
    (might want to implement bump start)

    4: strobe 1 intensity
    5: strobe 1 rate
    6: strobe 2 intensity
    7: strobe 2 rate
    """

    def __init__(self, dmx_addr):
        self.dmx_addr = dmx_addr

        # bipolar
        self.ball_rotation = 0.0

        self.ball_start = False

        # unipolar
        self.color_rotation = 0.0
        self.color_start = False

        self.strobe_1_state = False
        self.strobe_2_state = False

        # unipolar
        self.strobe_1_intensity = 0.0
        self.strobe_2_intensity = 0.0
        self.strobe_1_rate = 0.0
        self.strobe_2_rate = 0.0

    def update(self, timestep):
        pass

    def _render_ball_rotation(self):
        speed = abs(self.ball_rotation)
        direction = speed >= 0.0
        if self.ball_start and speed < 0.2:
            speed = 0.2
        dmx_speed = unit_float_to_range(0, 255, speed)
        dmx_direction = 0 if direction else 255
        return dmx_speed, dmx_direction

    def _render_color_rotation(self):
        if self.color_start and self.color_rotation < 0.2:
            speed = 0.2
        else:
            speed = self.color_rotation
        return unit_float_to_range(0, 255, speed)

    def render(self, dmx_univ):
        """Render this Comet into a DMX universe."""
        start = self.dmx_addr

        # render the shutter value
        dmx_univ[start:start+2] = self._render_ball_rotation()
        dmx_univ[start+2] = self._render_color_rotation()
        dmx_univ[start+3:start+5] = _render_strobe(
            self.strobe_1_state, self.strobe_1_intensity, self.strobe_1_rate)
        dmx_univ[start+5:start+7] = _render_strobe(
            self.strobe_2_state, self.strobe_2_intensity, self.strobe_2_rate)

def reflective_control(name, preprocessor=None):
    """Define a control that uses reflection to

# control actions
def base_rotation(venus, speed):
    """bipolar float"""
    venus.base_rotation = speed

def cradle_rotation(venus, speed):
    """unipolar float"""
    venus.cradle_rotation = speed

def head_rotation(venus, speed):
    """bipolar float"""
    venus.head_rotation = speed

def color_rotation(venus, speed):
    """bipolar float"""
    venus.color_rotation = speed

# control mapping
control_map = {
    BaseRotation: base_rotation,
    CradleRotation: cradle_rotation,
    HeadRotation: head_rotation,
    ColorRotation: color_rotation,}


def setup_controls(cont):

    # make groups
    cont.create_control_group('Controls')
    cont.create_control_group('Debug')

    # add controls
    cont.create_simple_control('Controls', BaseRotation, BaseRotation)
    cont.create_simple_control('Controls', CradleRotation, CradleRotation)
    cont.create_simple_control('Controls', HeadRotation, HeadRotation)
    cont.create_simple_control('Controls', ColorRotation, ColorRotation)