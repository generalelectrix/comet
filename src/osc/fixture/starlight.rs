use rosc::OscMessage;

use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::starlight::{Starlight, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::fixture::generic::map_strobe;
use crate::osc::radio_button::EnumRadioButton;
use crate::osc::{ignore_payload, HandleStateChange};
use crate::osc::{ControlMap, MapControls};
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = "Starlight";

impl MapControls for Starlight {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::Starlight;
        use StateChange::*;
        map.add_unipolar(GROUP, "Dimmer", |v| Starlight(Dimmer(v)));
        map.add_bipolar(GROUP, "Rotation", |v| {
            Starlight(Rotation(bipolar_fader_with_detent(v)))
        });
        map_strobe(map, GROUP, "Strone", &wrap_strobe);
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> FixtureControlMessage {
    FixtureControlMessage::Starlight(StateChange::Strobe(sc))
}

impl HandleStateChange<StateChange> for Starlight {
    fn emit_state_change<S>(_sc: StateChange, _send: &mut S)
    where
        S: FnMut(OscMessage),
    {
        // FIXME: implement talkback
    }
}
