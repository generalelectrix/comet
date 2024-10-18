use super::generic::map_strobe;
use crate::fixture::colordynamic::{Colordynamic, ControlMessage, StateChange};
use crate::fixture::generic::GenericStrobeStateChange;

use crate::fixture::PatchAnimatedFixture;

use crate::osc::basic_controls::{button, Button};
use crate::osc::GroupControlMap;
use crate::osc::HandleOscStateChange;
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = Colordynamic::NAME.0;

const SHUTTER_OPEN: Button = button(GROUP, "ShutterOpen");
const COLOR_ROTATION_ON: Button = button(GROUP, "ColorRotationOn");

impl Colordynamic {
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;
        SHUTTER_OPEN.map_state(map, ShutterOpen);
        map_strobe(map, "Strobe", &wrap_strobe);

        COLOR_ROTATION_ON.map_state(map, ColorRotationOn);
        map.add_unipolar("ColorRotationSpeed", ColorRotationSpeed);
        map.add_unipolar("ColorPosition", ColorPosition);
        map.add_bipolar("FiberRotation", |v| {
            FiberRotation(bipolar_fader_with_detent(v))
        });
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessage {
    StateChange::Strobe(sc)
}

impl HandleOscStateChange<StateChange> for Colordynamic {
    fn emit_osc_state_change<S>(_sc: StateChange, _send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        // FIXME no talkback
    }
}
