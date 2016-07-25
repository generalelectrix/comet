
BaseRotation = 'BaseRotation'
CradleRotation = 'CradleRotation'
HeadRotation = 'HeadRotation'
ColorRotation = 'ColorRotation'

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