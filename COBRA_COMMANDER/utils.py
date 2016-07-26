def unit_float_to_range(start, end, value):
    return int((end-start)*value)+start

# preprocessor helper functions
def ignore_all_but_1(value):
    return value if value == 1.0 else None

def quadratic_fader(value):
    return value**2

def quartic_fader(value):
    return value**4