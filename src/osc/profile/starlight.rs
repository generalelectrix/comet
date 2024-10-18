use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::starlight::{ControlMessage, Starlight, StateChange};

use crate::fixture::PatchAnimatedFixture;
use crate::osc::profile::generic::map_strobe;
use crate::osc::GroupControlMap;
use crate::osc::HandleOscStateChange;
use crate::util::bipolar_fader_with_detent;

const GROUP: &str = Starlight::NAME.0;

impl Starlight {
    fn group(&self) -> &'static str {
        GROUP
    }
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;
        map.add_unipolar("Dimmer", Dimmer);
        map.add_bipolar("Rotation", |v| Rotation(bipolar_fader_with_detent(v)));
        map_strobe(map, "Strobe", &wrap_strobe);
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessage {
    StateChange::Strobe(sc)
}

impl HandleOscStateChange<StateChange> for Starlight {
    fn emit_osc_state_change<S>(_sc: StateChange, _send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        // FIXME: implement talkback
    }
}
