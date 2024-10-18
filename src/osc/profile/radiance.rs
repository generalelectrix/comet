use crate::fixture::radiance::{ControlMessage, Radiance, StateChange};

use crate::osc::{GroupControlMap, HandleOscStateChange};

impl Radiance {
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;
        map.add_unipolar("Haze", Haze);
        map.add_unipolar("Fan", Fan);
    }
}

impl HandleOscStateChange<StateChange> for Radiance {}
