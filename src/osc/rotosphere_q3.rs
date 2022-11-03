use rosc::OscMessage;

use super::ControlMap;
use crate::fixture::ControlMessage::{self as ShowControlMessage, RotosphereQ3};
use crate::generic::GenericStrobeStateChange;
use crate::osc::generic::map_strobe;
use crate::rotosphere_q3::StateChange;
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "RotosphereQ3";

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
    use StateChange::*;

    map.add_unipolar(GROUP, "Red", |v| RotosphereQ3(Red(v)));
    map.add_unipolar(GROUP, "Green", |v| RotosphereQ3(Green(v)));
    map.add_unipolar(GROUP, "Blue", |v| RotosphereQ3(Blue(v)));
    map.add_unipolar(GROUP, "White", |v| RotosphereQ3(White(v)));
    map_strobe(map, GROUP, "Strobe", &wrap_strobe);
    map.add_bipolar(GROUP, "Rotation", |v| {
        RotosphereQ3(Rotation(bipolar_fader_with_detent(v)))
    });
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ShowControlMessage {
    RotosphereQ3(StateChange::Strobe(sc))
}

pub fn handle_state_change<S>(_sc: StateChange, _send: &mut S)
where
    S: FnMut(OscMessage),
{
    // No controls with talkback.
}
