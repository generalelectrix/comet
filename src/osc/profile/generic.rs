use crate::fixture::generic::GenericStrobeStateChange;

use crate::fixture::ControlMessagePayload;
use crate::osc::GroupControlMap;

pub fn map_strobe<F>(map: &mut GroupControlMap<ControlMessagePayload>, name: &str, wrap: &'static F)
where
    F: Fn(GenericStrobeStateChange) -> ControlMessagePayload + 'static,
{
    map.add_bool(&format!("{}On", name), move |v| {
        wrap(GenericStrobeStateChange::On(v))
    });
    map.add_unipolar(&format!("{}Rate", name), move |v| {
        wrap(GenericStrobeStateChange::Rate(v))
    });
}
