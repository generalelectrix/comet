use crate::fixture::venus::{ControlMessage, StateChange, Venus};

use crate::fixture::PatchFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{GroupControlMap, HandleOscStateChange};
use crate::util::bipolar_fader_with_detent;
use crate::util::unipolar_fader_with_detent;

const GROUP: &str = Venus::NAME.0;

const LAMP_ON: Button = button(GROUP, "LampControl");

impl Venus {
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;

        map.add_bipolar("BaseRotation", |v| {
            BaseRotation(bipolar_fader_with_detent(v))
        });
        map.add_unipolar("CradleMotion", |v| {
            CradleMotion(unipolar_fader_with_detent(v))
        });
        map.add_bipolar("HeadRotation", |v| {
            HeadRotation(bipolar_fader_with_detent(v))
        });
        map.add_bipolar("ColorRotation", |v| {
            ColorRotation(bipolar_fader_with_detent(v))
        });
        LAMP_ON.map_state(map, LampOn);
    }
}

impl HandleOscStateChange<StateChange> for Venus {}
