use crate::fixture::radiance::{Radiance, StateChange};
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchFixture;
use crate::osc::{GroupControlMap, HandleOscStateChange, MapControls};

const GROUP: &str = Radiance::NAME.0;

impl MapControls for Radiance {
    fn group(&self) -> &'static str {
        GROUP
    }
    fn map_controls(&self, map: &mut GroupControlMap<ControlMessagePayload>) {
        use StateChange::*;
        map.add_unipolar("Haze", |v| ControlMessagePayload::fixture(Haze(v)));
        map.add_unipolar("Fan", |v| ControlMessagePayload::fixture(Fan(v)));
    }
}

impl HandleOscStateChange<StateChange> for Radiance {}
