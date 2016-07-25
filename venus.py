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