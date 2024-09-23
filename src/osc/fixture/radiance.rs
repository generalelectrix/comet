use crate::fixture::radiance::{Radiance, StateChange};
use crate::fixture::ControlMessagePayload;
use crate::osc::{ControlMap, HandleStateChange, MapControls};

const GROUP: &str = "Radiance";

impl MapControls for Radiance {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        use StateChange::*;
        map.add_unipolar(GROUP, "Haze", |v| ControlMessagePayload::fixture(Haze(v)));
        map.add_unipolar(GROUP, "Fan", |v| ControlMessagePayload::fixture(Fan(v)));
    }
}

impl HandleStateChange<StateChange> for Radiance {}
