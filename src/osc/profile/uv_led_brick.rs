use crate::fixture::uv_led_brick::{ControlMessage, StateChange, UvLedBrick};

use crate::fixture::PatchAnimatedFixture;
use crate::osc::{GroupControlMap, HandleOscStateChange};

const GROUP: &str = UvLedBrick::NAME.0;

impl UvLedBrick {
    fn group(&self) -> &'static str {
        GROUP
    }
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        map.add_unipolar("Level", |x| x);
    }
}

impl HandleOscStateChange<StateChange> for UvLedBrick {}
