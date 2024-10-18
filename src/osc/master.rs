//! Control mappings for show-level controls.

use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::ControlMessagePayload;
use crate::master::{MasterControls, StateChange};

use super::basic_controls::{button, Button};
use super::profile::generic::map_strobe;
use super::{GroupControlMap, HandleOscStateChange, MapControls};

const GROUP: &str = "Master";

const USE_MASTER_STROBE_RATE: Button = button(GROUP, "UseMasterStrobeRate");
const REFRESH_UI: Button = button(GROUP, "RefreshUI");

impl MapControls for MasterControls {
    fn group(&self) -> &'static str {
        GROUP
    }

    fn map_controls(&self, map: &mut GroupControlMap<ControlMessagePayload>) {
        map_strobe(map, "Strobe", &wrap_strobe);
        USE_MASTER_STROBE_RATE.map_state(map, |v| {
            ControlMessagePayload::Master(StateChange::UseMasterStrobeRate(v))
        });
        REFRESH_UI.map_trigger(map, || ControlMessagePayload::RefreshUI)
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        Default::default()
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessagePayload {
    ControlMessagePayload::Master(StateChange::Strobe(sc))
}

impl HandleOscStateChange<StateChange> for MasterControls {}
