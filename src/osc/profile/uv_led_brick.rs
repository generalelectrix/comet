use crate::fixture::uv_led_brick::{ControlMessage, StateChange, UvLedBrick};

use crate::osc::{GroupControlMap, HandleOscStateChange};

impl UvLedBrick {
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        map.add_unipolar("Level", |x| x);
    }
}

impl HandleOscStateChange<StateChange> for UvLedBrick {}
