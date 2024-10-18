use crate::fixture::generic::GenericStrobeStateChange;

use crate::fixture::ControlMessagePayload;
use crate::osc::ControlMap;

pub fn map_strobe<F>(
    map: &mut ControlMap<ControlMessagePayload>,
    group: &str,
    name: &str,
    wrap: &'static F,
) where
    F: Fn(GenericStrobeStateChange) -> ControlMessagePayload + 'static,
{
    map.add_bool(group, &format!("{}On", name), move |v| {
        wrap(GenericStrobeStateChange::On(v))
    });
    map.add_unipolar(group, &format!("{}Rate", name), move |v| {
        wrap(GenericStrobeStateChange::Rate(v))
    });
}
