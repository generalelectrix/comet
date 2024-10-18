//! Control mappings for show-level controls.

use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::ControlMessagePayload;
use crate::master::{ControlMessage, MasterControls, StateChange};

use super::basic_controls::{button, Button};
use super::profile::generic::map_strobe;
use super::{GroupControlMap, HandleOscStateChange, MapControls};

pub(crate) const GROUP: &str = "Master";

const USE_MASTER_STROBE_RATE: Button = button(GROUP, "UseMasterStrobeRate");
const REFRESH_UI: Button = button(GROUP, "RefreshUI");

impl MasterControls {
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        map_strobe(map, "Strobe", &wrap_strobe);
        USE_MASTER_STROBE_RATE.map_state(map, |v| {
            ControlMessage::State(StateChange::UseMasterStrobeRate(v))
        });
        REFRESH_UI.map_trigger(map, || ControlMessage::RefreshUI)
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessage {
    ControlMessage::State(StateChange::Strobe(sc))
}

impl HandleOscStateChange<StateChange> for MasterControls {}
