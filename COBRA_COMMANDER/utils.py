from math import copysign

def unit_float_to_range(start, end, value):
    return int((end-start)*value)+start

# preprocessor helper functions
def ignore_all_but_1(value):
    return value if value == 1.0 else None

def quadratic_fader(value):
    return value**2

def quartic_fader(value):
    return value**4

def unipolar_fader_with_detent(value):
    """Coerce the bottom 5% of the fader range to be a hard 0, and rescale the rest."""
    if value < 0.05:
        return 0.0
    else:
        return (value - 0.05) / 0.95

def bipolar_fader_with_detent(value):
    """Coerce the center 5% of the fader range to be a hard 0, and rescale the rest."""
    if value < 0.0:
        if value > -0.05:
            return 0.0
        else:
            return (value + 0.05) / 0.95
    else:
        if value < 0.05:
            return 0.0
        else:
            return (value - 0.05) / 0.95


class RampingParameter (object):
    """A fixture parameter that ramps to its setpoint at a finite rate."""
    def __init__(self, initial_value=0.0, ramp_rate=1.0):
        """Args:
            initial_value: where the parameter should start
            ramp_rate: units per second for the parameter to ramp
        """
        self.target = initial_value
        self.current = initial_value
        # ramp rate is internally in units per ms
        self.ramp_rate = ramp_rate / 1000.

    def update(self, timestep):
        target, current = self.target, self.current
        delta = target - current
        step = copysign(self.ramp_rate * timestep, delta)
        if abs(step) > abs(delta):
            self.current = target
        else:
            self.current += step