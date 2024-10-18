use crate::fixture::venus::{StateChange, Venus};

use crate::fixture::PatchFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{GroupControlMap, HandleOscStateChange};
use crate::util::bipolar_fader_with_detent;
use crate::util::unipolar_fader_with_detent;

const GROUP: &str = Venus::NAME.0;

const LAMP_ON: Button = button(GROUP, "LampControl");

impl Venus {
    fn group(&self) -> &'static str {
        GROUP
    }
    fn map_controls(&self, map: &mut GroupControlMap<ControlMessagePayload>) {
        use StateChange::*;

        map.add_bipolar("BaseRotation", |v| {
            ControlMessagePayload::fixture(BaseRotation(bipolar_fader_with_detent(v)))
        });
        map.add_unipolar("CradleMotion", |v| {
            ControlMessagePayload::fixture(CradleMotion(unipolar_fader_with_detent(v)))
        });
        map.add_bipolar("HeadRotation", |v| {
            ControlMessagePayload::fixture(HeadRotation(bipolar_fader_with_detent(v)))
        });
        map.add_bipolar("ColorRotation", |v| {
            ControlMessagePayload::fixture(ColorRotation(bipolar_fader_with_detent(v)))
        });
        LAMP_ON.map_state(map, |v| ControlMessagePayload::fixture(LampOn(v)));
    }
}

impl HandleOscStateChange<StateChange> for Venus {}
