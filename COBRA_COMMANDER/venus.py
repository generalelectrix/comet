"""Abstraction layer on top of the DMX interface to the RA venus."""

class Venus(object):

    def __init__(self, dmx_addr):
        """Create a new wrapper for a Venus.

        All controls are bipolar or unipolar floats.
        """
        self.dmx_addr = dmx_addr

        self.base_rotation = 0.0
        self.cradle_rotation = 0.0
        self.head_rotation = 0.0
        self.color_rotation = 0.0

    def render(self, dmx_univ):
        """Render this Comet into a DMX universe."""
        #TODO
        pass

"""
What do you think?

======================================

DMX profile Venus

Motor 1 is base motor
Motor 2 is crescent translate motor
Motor 3 is saucer off axis rotate motor
Motor 4 is color carousel

Motor direction is split at 127
Lamp on/off is split at 127 (high is on)

1 - Motor 1 Dir
2 - Motor 1 Speed
3 - Motor 2 Dir
4 - Motor 2 Speed
5 - Motor 3 Dir
6 - Motor 3 Speed
7 - Motor 4 Dir
8 - Motor 4 Speed
9 - Lamp Control
"""