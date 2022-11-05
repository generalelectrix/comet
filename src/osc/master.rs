//! Control mappings for show-level controls.

use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::FixtureControlMessage;
use crate::master::{MasterControls, StateChange};

use super::fixture::generic::map_strobe;
use super::{ControlMap, HandleStateChange, MapControls};

const GROUP: &str = "Global";

impl MapControls for MasterControls {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        map_strobe(map, GROUP, "Strobe", &wrap_strobe);
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> FixtureControlMessage {
    FixtureControlMessage::Master(StateChange::Strobe(sc))
}

impl HandleStateChange<StateChange> for MasterControls {}
