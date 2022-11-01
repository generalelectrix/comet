use rosc::OscMessage;

use super::ControlMap;
use crate::fixture::ControlMessage::{self as ShowControlMessage, Venus};
use crate::util::bipolar_fader_with_detent;
use crate::util::unipolar_fader_with_detent;
use crate::venus::StateChange;

const CONTROLS: &str = "Controls";
const LAMP: &str = "Lamp";

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
    use StateChange::*;

    map.add_bipolar(CONTROLS, "BaseRotation", |v| {
        Venus(BaseRotation(bipolar_fader_with_detent(v)))
    });
    map.add_unipolar(CONTROLS, "CradleMotion", |v| {
        Venus(CradleMotion(unipolar_fader_with_detent(v)))
    });
    map.add_bipolar(CONTROLS, "HeadRotation", |v| {
        Venus(HeadRotation(bipolar_fader_with_detent(v)))
    });
    map.add_bipolar(CONTROLS, "ColorRotation", |v| {
        Venus(ColorRotation(bipolar_fader_with_detent(v)))
    });
    map.add_bool(LAMP, "LampControl", |v| Venus(LampOn(v)));
}

pub fn handle_state_change<S>(_: StateChange, _: &mut S)
where
    S: FnMut(OscMessage),
{
    // No venus OSC controls use talkback.
}
