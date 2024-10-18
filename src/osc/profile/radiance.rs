use crate::fixture::radiance::{ControlMessage, Radiance, StateChange};

use crate::fixture::PatchFixture;
use crate::osc::{GroupControlMap, HandleOscStateChange};

const GROUP: &str = Radiance::NAME.0;

impl Radiance {
    fn group(&self) -> &'static str {
        GROUP
    }
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;
        map.add_unipolar("Haze", |v| Haze(v));
        map.add_unipolar("Fan", |v| Fan(v));
    }
}

impl HandleOscStateChange<StateChange> for Radiance {}
