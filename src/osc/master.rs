//! Control mappings for show-level controls.

use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::FixtureControlMessage;
use crate::master::{MasterControls, StateChange};

use super::fixture::generic::map_strobe;
use super::{ControlMap, HandleStateChange, MapControls};

const GROUP: &str = "Master";

impl MapControls for MasterControls {
    fn group(&self) -> &'static str {
        GROUP
    }
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        map_strobe(map, GROUP, "Strobe", &wrap_strobe);
        map.add_bool(GROUP, "UseMasterStrobeRate", |v| {
            FixtureControlMessage::Master(StateChange::UseMasterStrobeRate(v))
        });
        map.add_bool(GROUP, "AutopilotOn", |v| {
            FixtureControlMessage::Master(StateChange::AutopilotOn(v))
        });
        map.add_bool(GROUP, "AutopilotSoundActive", |v| {
            FixtureControlMessage::Master(StateChange::AutopilotSoundActive(v))
        });
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> FixtureControlMessage {
    FixtureControlMessage::Master(StateChange::Strobe(sc))
}

impl HandleStateChange<StateChange> for MasterControls {}
