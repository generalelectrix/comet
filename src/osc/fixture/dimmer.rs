use crate::fixture::dimmer::{Dimmer, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::{ControlMap, HandleStateChange, MapControls};

const GROUP: &str = "Dimmer";

impl MapControls for Dimmer {
    fn group(&self) -> &'static str {
        GROUP
    }
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        map.add_unipolar(GROUP, "Level", FixtureControlMessage::Dimmer);
    }
}

impl HandleStateChange<StateChange> for Dimmer {}
