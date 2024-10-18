use crate::fixture::dimmer::{Dimmer, StateChange};

use crate::fixture::PatchAnimatedFixture;
use crate::osc::{GroupControlMap, HandleOscStateChange};

const GROUP: &str = Dimmer::NAME.0;

impl Dimmer {
    fn group(&self) -> &'static str {
        GROUP
    }

    fn map_controls(&self, map: &mut GroupControlMap<ControlMessagePayload>) {
        map.add_unipolar("Level", ControlMessagePayload::fixture);
    }
}

impl HandleOscStateChange<StateChange> for Dimmer {
    fn emit_osc_state_change<S>(_sc: StateChange, _send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
    }
}
