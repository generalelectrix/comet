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
