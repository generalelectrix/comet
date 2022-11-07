use crate::fixture::dimmer::{Dimmer, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::{ControlMap, HandleStateChange, MapControls};

const GROUP: &str = "Dimmer";

impl MapControls for Dimmer {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        map.add_unipolar(GROUP, "Level", |v| FixtureControlMessage::Dimmer(v));
    }
}

impl HandleStateChange<StateChange> for Dimmer {}
