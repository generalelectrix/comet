
from utils import unit_float_to_range, bipolar_fader_with_detent, unipolar_fader_with_detent

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

def build_lumasphere_controls():
    """Create the lumasphere control mappings, returning the control map and setup_controls function."""

    control_map = {}

    def reflective_control(name, preprocessor=None):
        """Define a control that uses reflection to push a value."""
        if preprocessor is not None:
            control = lambda fixture, value: setattr(fixture, name, preprocessor(value))
        else:
            control = lambda fixture, value: setattr(fixture, name, value)
        control_map[name] = control

    reflective_control('ball_rotation', preprocessor=bipolar_fader_with_detent)
    reflective_control('ball_start', preprocessor=bool)
    reflective_control('color_rotation', preprocessor=unipolar_fader_with_detent)
    reflective_control('color_start', preprocessor=bool)
    reflective_control('strobe_1_state', preprocessor=bool)
    reflective_control('strobe_2_state', preprocessor=bool)
    reflective_control('strobe_1_intensity')
    reflective_control('strobe_2_intensity')
    reflective_control('strobe_1_rate')
    reflective_control('strobe_2_rate')

    def setup_controls(cont):
        for name in control_map.iterkeys():
            cont.create_simple_control('Lumasphere', name, name)

    return control_map, setup_controls
