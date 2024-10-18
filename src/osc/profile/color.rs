use crate::fixture::color::{Color, StateChange};

use crate::fixture::PatchAnimatedFixture;
use crate::osc::{GroupControlMap, HandleOscStateChange};

const GROUP: &str = Color::NAME.0;

impl Color {
    fn group(&self) -> &'static str {
        GROUP
    }

    fn map_controls(&self, map: &mut GroupControlMap<ControlMessage>) {
        map_color(map, &wrap_color);
    }
}

impl HandleOscStateChange<StateChange> for Color {}

fn wrap_color(sc: StateChange) -> ControlMessagePayload {
    sc
}

pub fn map_color<F>(map: &mut GroupControlMap<ControlMessage>, wrap: &'static F)
where
    F: Fn(StateChange) -> ControlMessagePayload + 'static,
{
    map.add_phase("Hue", move |v| wrap(StateChange::Hue(v)));
    map.add_unipolar("Sat", move |v| wrap(StateChange::Sat(v)));
    map.add_unipolar("Val", move |v| wrap(StateChange::Val(v)));
}
