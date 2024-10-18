use crate::fixture::dimmer::{ControlMessage, Dimmer, StateChange};

use crate::osc::{GroupControlMap, HandleOscStateChange};

impl Dimmer {
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        map.add_unipolar("Level", |x| x);
    }
}

impl HandleOscStateChange<StateChange> for Dimmer {
    fn emit_osc_state_change<S>(_sc: StateChange, _send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
    }
}
