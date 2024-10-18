use crate::fixture::uv_led_brick::{StateChange, UvLedBrick};

use crate::fixture::PatchAnimatedFixture;
use crate::osc::{GroupControlMap, HandleOscStateChange};

const GROUP: &str = UvLedBrick::NAME.0;

impl UvLedBrick {
    fn group(&self) -> &'static str {
        GROUP
    }
    fn map_controls(&self, map: &mut GroupControlMap<ControlMessagePayload>) {
        map.add_unipolar("Level", ControlMessagePayload::fixture);
    }
}

impl HandleOscStateChange<StateChange> for UvLedBrick {}
