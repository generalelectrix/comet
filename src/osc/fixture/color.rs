use crate::fixture::color::{Color, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::{ControlMap, HandleStateChange, MapControls};

const GROUP: &str = "Color";

impl MapControls for Color {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        map_color(map, GROUP, &wrap_color);
    }
}

impl HandleStateChange<StateChange> for Color {}

fn wrap_color(sc: StateChange) -> FixtureControlMessage {
    FixtureControlMessage::Color(sc)
}

pub fn map_color<F>(map: &mut ControlMap<FixtureControlMessage>, group: &str, wrap: &'static F)
where
    F: Fn(StateChange) -> FixtureControlMessage + 'static,
{
    map.add_phase(group, "Hue", move |v| wrap(StateChange::Hue(v)));
    map.add_unipolar(group, "Sat", move |v| wrap(StateChange::Sat(v)));
    map.add_unipolar(group, "Val", move |v| wrap(StateChange::Val(v)));
}
