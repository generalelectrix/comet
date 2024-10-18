use crate::fixture::uv_led_brick::{StateChange, UvLedBrick};
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchAnimatedFixture;
use crate::osc::{GroupControlMap, HandleOscStateChange, MapControls};

const GROUP: &str = "UvLedBrick";

impl MapControls for UvLedBrick {
    fn group(&self) -> &'static str {
        GROUP
    }
    fn map_controls(&self, map: &mut GroupControlMap<ControlMessagePayload>) {
        map.add_unipolar("Level", ControlMessagePayload::fixture);
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        vec![(GROUP.to_string(), Self::NAME)]
    }
}

impl HandleOscStateChange<StateChange> for UvLedBrick {}
