
(Shutter,
 Strobe,
 StrobeRate,
 ShutterSoundActive,
 SelectMacro,
 Mspeed,
 Reset,
 StepForwards,
 StepBackwards,
 TrigSoundActive,
 AutoStep,
 AutoStepRate) = range(12)

# control actions
def shutter_state(comet, state):
    """bool"""
    comet.shutter_open = state

def strobe_state(comet, state):
    """bool"""
    comet.strobing = state

def strobe_rate(comet, rate):
    """float in [0,1]"""
    comet.strobe_rate = rate

def shutter_sound_active(comet, state):
    """bool"""
    comet.shutter_sound_active = state

def select_macro_pattern(comet, pattern):
    """int in [0,9]"""
    comet.macro_pattern = pattern

def mspeed(comet, mspeed):
    """float in [0,1]"""
    comet.mirror_speed = mspeed

def reset(comet, reset):
    """bool"""
    comet.reset = reset

def step_forwards(comet, _):
    comet.trigger_state.step_forwards()

def step_backwards(comet, _):
    comet.trigger_state.step_forwards()

def trigger_sound_active(comet, state):
    """bool"""
    comet.trigger_state.music_trigger = state

def auto_step(comet, state):
    """bool"""
    comet.trigger_state.auto_step = state

def auto_step_rate(comet, rate):
    """float in [0,1]"""
    comet.trigger_state.auto_step_rate = rate


# control mapping
control_map = {
    Shutter: shutter_state,
    Strobe: strobe_state,
    StrobeRate: strobe_rate,
    ShutterSoundActive: shutter_sound_active,
    SelectMacro: select_macro_pattern,
    Mspeed: mspeed,
    Reset: reset,
    StepForwards: step_forwards,
    StepBackwards: step_backwards,
    TrigSoundActive: trigger_sound_active,
    AutoStep: auto_step,
    AutoStepRate: auto_step_rate,}