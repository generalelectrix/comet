use crate::fixture::uv_led_brick::{StateChange, UvLedBrick};
use crate::fixture::ControlMessagePayload;
use crate::osc::{ControlMap, HandleStateChange, MapControls};

const GROUP: &str = "UvLedBrick";

impl MapControls for UvLedBrick {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        map.add_unipolar(GROUP, "Level", ControlMessagePayload::fixture);
    }
}

impl HandleStateChange<StateChange> for UvLedBrick {}
