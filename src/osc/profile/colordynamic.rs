use super::generic::map_strobe;
use crate::fixture::colordynamic::{Colordynamic, StateChange};
use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchAnimatedFixture;

use crate::osc::basic_controls::{button, Button};
use crate::osc::HandleOscStateChange;
use crate::osc::{GroupControlMap, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "Colordynamic";

const SHUTTER_OPEN: Button = button(GROUP, "ShutterOpen");
const COLOR_ROTATION_ON: Button = button(GROUP, "ColorRotationOn");

impl MapControls for Colordynamic {
    fn group(&self) -> &'static str {
        GROUP
    }

    fn map_controls(&self, map: &mut GroupControlMap<ControlMessagePayload>) {
        use StateChange::*;
        SHUTTER_OPEN.map_state(map, |v| ControlMessagePayload::fixture(ShutterOpen(v)));
        map_strobe(map, "Strobe", &wrap_strobe);

        COLOR_ROTATION_ON.map_state(map, |v| ControlMessagePayload::fixture(ColorRotationOn(v)));
        map.add_unipolar("ColorRotationSpeed", |v| {
            ControlMessagePayload::fixture(ColorRotationSpeed(v))
        });
        map.add_unipolar("ColorPosition", |v| {
            ControlMessagePayload::fixture(ColorPosition(v))
        });
        map.add_bipolar("FiberRotation", |v| {
            ControlMessagePayload::fixture(FiberRotation(bipolar_fader_with_detent(v)))
        });
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        vec![(GROUP.to_string(), Self::NAME)]
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessagePayload {
    ControlMessagePayload::fixture(StateChange::Strobe(sc))
}

impl HandleOscStateChange<StateChange> for Colordynamic {
    fn emit_osc_state_change<S>(_sc: StateChange, _send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        // FIXME no talkback
    }
}
