use crate::fixture::color::{Color, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::{ControlMap, HandleStateChange, MapControls};

const GROUP: &str = "Color";

impl MapControls for Color {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::Color;
        use StateChange::*;
        map.add_phase(GROUP, "Hue", |v| Color(Hue(v)));
        map.add_unipolar(GROUP, "Sat", |v| Color(Sat(v)));
        map.add_unipolar(GROUP, "Val", |v| Color(Val(v)));
    }
}

impl HandleStateChange<StateChange> for Color {}
