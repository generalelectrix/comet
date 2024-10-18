use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::lumasphere::StrobeStateChange;
use crate::fixture::lumasphere::{ControlMessage, Lumasphere, StateChange};

use crate::fixture::PatchFixture;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{GroupControlMap, HandleOscStateChange};
use crate::util::bipolar_fader_with_detent;
use crate::util::unipolar_fader_with_detent;

const GROUP: &str = Lumasphere::NAME.0;

const BALL_START: Button = button(GROUP, "ball_start");
const COLOR_START: Button = button(GROUP, "color_start");

impl Lumasphere {
    fn group(&self) -> &'static str {
        GROUP
    }
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;
        map.add_unipolar("lamp_1_intensity", |v| {
            Lamp1Intensity(unipolar_fader_with_detent(v))
        });
        map.add_unipolar("lamp_2_intensity", |v| {
            Lamp2Intensity(unipolar_fader_with_detent(v))
        });

        map.add_bipolar("ball_rotation", |v| {
            BallRotation(bipolar_fader_with_detent(v))
        });
        BALL_START.map_state(map, BallStart);

        map.add_unipolar("color_rotation", |v| {
            ColorRotation(unipolar_fader_with_detent(v))
        });
        COLOR_START.map_state(map, ColorStart);
        map_strobe(map, 1, Strobe1);
        map_strobe(map, 2, Strobe2);
    }
}

fn map_strobe<W>(map: &mut GroupControlMap<ControlMessage>, index: u8, wrap: W)
where
    W: Fn(StrobeStateChange) -> ControlMessage + 'static + Copy,
{
    use GenericStrobeStateChange::*;
    use StrobeStateChange::*;
    map.add_bool(&format!("strobe_{}_state", index), move |v| {
        wrap(State(On(v)))
    });
    map.add_unipolar(&format!("strobe_{}_rate", index), move |v| {
        wrap(State(Rate(v)))
    });
    map.add_unipolar(&format!("strobe_{}_intensity", index), move |v| {
        wrap(Intensity(v))
    });
}

impl HandleOscStateChange<StateChange> for Lumasphere {}
