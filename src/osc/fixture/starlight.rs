use rosc::OscMessage;

use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::starlight::{Starlight, StateChange};
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchAnimatedFixture;
use crate::osc::fixture::generic::map_strobe;
use crate::osc::HandleStateChange;
use crate::osc::{ControlMap, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "Starlight";

impl MapControls for Starlight {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        use StateChange::*;
        map.add_unipolar(GROUP, "Dimmer", |v| {
            ControlMessagePayload::fixture(Dimmer(v))
        });
        map.add_bipolar(GROUP, "Rotation", |v| {
            ControlMessagePayload::fixture(Rotation(bipolar_fader_with_detent(v)))
        });
        map_strobe(map, GROUP, "Strobe", &wrap_strobe);
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        vec![(GROUP.to_string(), Self::NAME)]
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessagePayload {
    ControlMessagePayload::fixture(StateChange::Strobe(sc))
}

impl HandleStateChange<StateChange> for Starlight {
    fn emit_state_change<S>(_sc: StateChange, _send: &mut S, _talkback: crate::osc::TalkbackMode)
    where
        S: FnMut(OscMessage),
    {
        // FIXME: implement talkback
    }
}
