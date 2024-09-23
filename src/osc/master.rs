//! Control mappings for show-level controls.

use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::ControlMessagePayload;
use crate::master::{MasterControls, StateChange};

use super::basic_controls::{button, Button};
use super::fixture::generic::map_strobe;
use super::{ControlMap, HandleStateChange, MapControls};

const GROUP: &str = "Master";

const USE_MASTER_STROBE_RATE: Button = button(GROUP, "UseMasterStrobeRate");
const AUTOPILOT_ON: Button = button(GROUP, "AutopilotOn");
const AUTOPILOT_SOUND_ACTIVE: Button = button(GROUP, "AutopilotSoundActive");
const REFRESH_UI: Button = button(GROUP, "RefreshUI");

impl MapControls for MasterControls {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        map_strobe(map, GROUP, "Strobe", &wrap_strobe);
        USE_MASTER_STROBE_RATE.map_state(map, |v| {
            ControlMessagePayload::Master(StateChange::UseMasterStrobeRate(v))
        });
        AUTOPILOT_ON.map_state(map, |v| {
            ControlMessagePayload::Master(StateChange::AutopilotOn(v))
        });
        AUTOPILOT_SOUND_ACTIVE.map_state(map, |v| {
            ControlMessagePayload::Master(StateChange::AutopilotSoundActive(v))
        });
        REFRESH_UI.map_trigger(map, || ControlMessagePayload::RefreshUI)
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessagePayload {
    ControlMessagePayload::Master(StateChange::Strobe(sc))
}

impl HandleStateChange<StateChange> for MasterControls {}
