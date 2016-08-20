"""Abstraction layer on top of the DMX interface to a Coemar Comet."""
from collections import namedtuple
from utils import unit_float_to_range, ignore_all_but_1, quadratic_fader

def bipolar_to_motor_speed(value):
    """Convert a bipolar float to swarmolon motor speed."""
    if value < 0.0:
        return unit_float_to_range(128, 5, abs(value))
    else:
        return unit_float_to_range(130, 255, value)

class Swarmolon(object):
    """Control abstraction for the MIGHTY SWARMOLON."""

    def __init__(self, dmx_addr):
        """Create a new wrapper for a swarmolon."""
        self.dmx_addr = dmx_addr

        # add strings like "red" to turn that LED on
        # options are
        # red, green, blue, amber, white
        self.led_state = dict(
            red=False, green=False, blue=False, amber=False, white=False)

        self.white_leds_on = False

        # int on 0 to 9
        self.white_led_program = 0

        # unipolar
        self.white_led_speed = 0.0

        self.red_laser = False
        self.green_laser = False

        # unipolar
        # not implemented right now

        # bipolar
        self.led_motor_speed = 0.0

        #bipolar
        self.laser_motor_speed = 0.0


    def update(self, timestep):
        pass

    def _render_led_state(self):
        colors = (color for color, state in self.led_state.iterkeys() if state)
        key = frozenset(colors)
        return LED_MAPPING[key]

    def _render_white_led_state(self):
        if not self.white_leds_on:
            return 0
        else:
            program_offset = 10*(self.white_led_program+1)
            speed_offset = unit_float_to_range(0, 9, self.white_led_speed)
            return program_offset + speed_offset

    LASER_DISPATCH = {
        (False, False): 0,
        (True, False): 49,
        (False, True): 89,
        (True, True): 255}

    def _render_laser_state(self):
        return self.LASER_DISPATCH[(self.red_laser, self.green_laser)]

    def render(self, dmx_univ):
        """Render this Comet into a DMX universe."""
        start = self.dmx_addr

        # render the shutter value
        dmx_univ[start] = 255 # DMX mode
        dmx_univ[start + 1] = self._render_led_state()
        dmx_univ[start + 2] = 0 # Auto program speed, not used here
        dmx_univ[start + 3] = 0 # LED strobe, not used here
        dmx_univ[start + 4] = self._render_white_led_state()
        dmx_univ[start + 5] = self._render_laser_state()
        dmx_univ[start + 6] = 0 # laser strobe, not used here
        dmx_univ[start + 7] = bipolar_to_motor_speed(self.led_motor_speed)
        dmx_univ[start + 8] = bipolar_to_motor_speed(self.laser_motor_speed)



# LED state dispatch
LED_STATES = [
    tuple(),
    ("red"),
    ("green"),
    ("blue"),
    ("amber"),
    ("white"),
    ("white", "red"),
    ("red", "green"),
    ("green", "blue"),
    ("blue", "amber"),
    ("amber", "white"),
    ("white", "green"),
    ("green", "amber"),
    ("amber", "red"),
    ("red", "blue"),
    ("blue", "white"),
    ("red", "green", "blue"),
    ("red", "green", "amber"),
    ("red", "green", "white"),
    ("red", "amber", "blue"),
    ("red", "white", "blue"),
    ("red", "amber", "white"),
    ("amber", "green", "blue"),
    ("blue", "green", "white"),
    ("amber", "green", "white"),
    ("amber", "white", "blue"),
    ("red", "green", "blue", "amber"),
    ("red", "green", "blue", "white"),
    ("green", "blue", "amber", "white"),
    ("red", "green", "amber", "white"),
    ("red", "blue", "amber", "white"),
    ("red", "green", "blue", "amber", "white")]

LED_MAPPING = {}
for i, line in enumerate(LED_STATES):
    contents = frozenset(item.strip() for item in line.split(', '))
    dmx_value = 14+(i*5)
    LED_MAPPING[contents] = dmx_value

# --- control mappings ---




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




















