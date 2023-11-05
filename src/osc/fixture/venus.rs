use crate::fixture::venus::{StateChange, Venus};
use crate::fixture::FixtureControlMessage;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{ControlMap, HandleStateChange, MapControls};
use crate::util::bipolar_fader_with_detent;
use crate::util::unipolar_fader_with_detent;

const CONTROLS: &str = "Controls";
const LAMP: &str = "Lamp";

const LAMP_ON: Button = button(LAMP, "LampControl");

impl MapControls for Venus {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::Venus;
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
        LAMP_ON.map_state(map, |v| Venus(LampOn(v)));
    }
}

impl HandleStateChange<StateChange> for Venus {}
