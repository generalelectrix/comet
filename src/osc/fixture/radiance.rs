use crate::fixture::radiance::{Radiance, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::{ControlMap, HandleStateChange, MapControls};

const GROUP: &str = "Radiance";

impl MapControls for Radiance {
    fn group(&self) -> &'static str {
        GROUP
    }
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::Radiance;
        use StateChange::*;
        map.add_unipolar(GROUP, "Haze", |v| Radiance(Haze(v)));
        map.add_unipolar(GROUP, "Fan", |v| Radiance(Fan(v)));
    }
}

impl HandleStateChange<StateChange> for Radiance {}
