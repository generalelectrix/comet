"""Abstraction layer on top of the DMX interface to the RA venus."""
from utils import (
    bipolar_fader_with_detent,
    unipolar_fader_with_detent,
    unit_float_to_range,
    RampingParameter)
import logging

def bipolar_to_dir_and_val(bipolar_val):
    if bipolar_val < 0.0:
        return 0, unit_float_to_range(0, 255, abs(bipolar_val))
    else:
        return 255, unit_float_to_range(0, 255, bipolar_val)

class Venus(object):

    def __init__(self, dmx_addr):
        """Create a new wrapper for a Venus."""
        self.dmx_addr = dmx_addr - 1

        self.base_rotation = RampingParameter()
        self.cradle_motion = RampingParameter()
        self.head_rotation = RampingParameter()
        self.color_rotation = RampingParameter()

        self.lamp_on = False

    def update(self, timestep):
        self.base_rotation.update(timestep)
        self.cradle_motion.update(timestep)
        self.head_rotation.update(timestep)
        self.color_rotation.update(timestep)

    def render(self, dmx_univ):
        """Render this Comet into a DMX universe."""
        dmx_addr = self.dmx_addr

        base_dir, base_val = bipolar_to_dir_and_val(self.base_rotation.current)
        cradle_val = unit_float_to_range(0, 255, self.cradle_motion.current)
        head_dir, head_val = bipolar_to_dir_and_val(self.head_rotation.current)
        col_dir, col_val = bipolar_to_dir_and_val(self.color_rotation.current)
        lamp_val = 255 if self.lamp_on else 0

        vals = (
            base_dir,
            base_val,
            cradle_val,
            head_dir,
            head_val,
            col_dir,
            col_val,
            lamp_val)

        #logging.debug("{}".format(vals))

        for offset, val in enumerate(vals):
            dmx_univ[dmx_addr+offset] = val

"""
DMX profile Venus

Motor 1 is base motor
Motor 2 is crescent translate motor
Motor 3 is saucer off axis rotate motor
Motor 4 is color carousel

Motor direction is split at 127
Lamp on/off is split at 127 (high is on)

1 - Motor 1 Dir
2 - Motor 1 Speed
3 - Motor 2 Speed
4 - Motor 3 Dir
5 - Motor 3 Speed
6 - Motor 4 Dir
7 - Motor 4 Speed
8 - Lamp Control
"""

# controls and control actions

(BaseRotation,
 CradleMotion,
 HeadRotation,
 ColorRotation,
 LampControl) = range(5)

# control actions
def base_rotation(venus, speed):
    venus.base_rotation.target = speed

def cradle_motion(venus, speed):
    venus.cradle_motion.target = speed

def head_rotation(venus, speed):
    venus.head_rotation.target = speed

def color_rotation(venus, speed):
    venus.color_rotation.target = speed

def lamp_on(venus, state):
    if state == 0.0:
        venus.lamp_on = False
    else:
        venus.lamp_on = True

# control mapping
control_map = {
    BaseRotation: base_rotation,
    CradleMotion: cradle_motion,
    HeadRotation: head_rotation,
    ColorRotation: color_rotation,
    LampControl: lamp_on}

controls_page = 'Controls'
lamp_page = 'Lamp'

def setup_controls(cont):

    # make groups
    cont.create_control_group(controls_page)
    cont.create_control_group(lamp_page)

    # add controls
    cont.create_simple_control(
        controls_page, 'BaseRotation', BaseRotation, bipolar_fader_with_detent)
    cont.create_simple_control(
        controls_page, 'CradleMotion', CradleMotion, unipolar_fader_with_detent)
    cont.create_simple_control(
        controls_page, 'HeadRotation', HeadRotation, bipolar_fader_with_detent)
    cont.create_simple_control(
        controls_page, 'ColorRotation', ColorRotation, bipolar_fader_with_detent)
    cont.create_simple_control(
        lamp_page, 'LampControl', LampControl)

