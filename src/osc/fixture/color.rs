use crate::fixture::color::{Color, StateChange};
use crate::fixture::ControlMessagePayload;
use crate::osc::{ControlMap, HandleStateChange, MapControls};

const GROUP: &str = "Color";

impl MapControls for Color {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        map_color(map, GROUP, &wrap_color);
    }
}

impl HandleStateChange<StateChange> for Color {}

fn wrap_color(sc: StateChange) -> ControlMessagePayload {
    ControlMessagePayload::fixture(sc)
}

pub fn map_color<F>(map: &mut ControlMap<ControlMessagePayload>, group: &str, wrap: &'static F)
where
    F: Fn(StateChange) -> ControlMessagePayload + 'static,
{
    map.add_phase(group, "Hue", move |v| wrap(StateChange::Hue(v)));
    map.add_unipolar(group, "Sat", move |v| wrap(StateChange::Sat(v)));
    map.add_unipolar(group, "Val", move |v| wrap(StateChange::Val(v)));
}
