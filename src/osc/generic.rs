use crate::generic::GenericStrobeStateChange;

use super::ControlMap;
use crate::fixture::ControlMessage;

pub fn map_strobe<F>(
    map: &mut ControlMap<ControlMessage>,
    group: &str,
    name: &str,
    wrap: &'static F,
) where
    F: Fn(GenericStrobeStateChange) -> ControlMessage + 'static,
{
    map.add_bool(group, format!("{}On", name), move |v| {
        wrap(GenericStrobeStateChange::On(v))
    });
    map.add_unipolar(group, format!("{}Rate", name), move |v| {
        wrap(GenericStrobeStateChange::Rate(v))
    });
}
