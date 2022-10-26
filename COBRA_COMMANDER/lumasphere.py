def build_lumasphere_controls():
    """Create the lumasphere control mappings, returning the control map and setup_controls function."""

    control_map = {}

    def reflective_control(name, preprocessor=None):
        """Define a control that uses reflection to push a value."""
        if preprocessor is not None:
            def control(fixture, value): return setattr(
                fixture, name, preprocessor(value))
        else:
            def control(fixture, value): return setattr(fixture, name, value)
        control_map[name] = control

    def ball_rotation(fixture, value):
        fixture.ball_rotation.target = bipolar_fader_with_detent(
            value) * 0.5  # let's not go too fast, OK?

    reflective_control('lamp_1_intensity',
                       preprocessor=unipolar_fader_with_detent)
    reflective_control('lamp_2_intensity',
                       preprocessor=unipolar_fader_with_detent)
    control_map['ball_rotation'] = ball_rotation
    reflective_control('ball_start', preprocessor=bool)
    reflective_control(
        'color_rotation', preprocessor=unipolar_fader_with_detent)
    reflective_control('color_start', preprocessor=bool)
    reflective_control('strobe_1_state', preprocessor=bool)
    reflective_control('strobe_2_state', preprocessor=bool)
    reflective_control('strobe_1_intensity')
    reflective_control('strobe_2_intensity')
    reflective_control('strobe_1_rate')
    reflective_control('strobe_2_rate')

    def setup_controls(cont):
        cont.create_control_group('Lumasphere')
        for name in control_map.iterkeys():
            cont.create_simple_control('Lumasphere', name, name)

    return control_map, setup_controls
