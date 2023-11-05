use crate::fixture::generic::GenericStrobeStateChange;
use crate::fixture::lumasphere::StrobeStateChange;
use crate::fixture::lumasphere::{Lumasphere, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::basic_controls::{button, Button};
use crate::osc::{ControlMap, HandleStateChange, MapControls};
use crate::util::bipolar_fader_with_detent;
use crate::util::unipolar_fader_with_detent;

const GROUP: &str = "Lumasphere";

const BALL_START: Button = button(GROUP, "ball_start");
const COLOR_START: Button = button(GROUP, "color_start");

impl MapControls for Lumasphere {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::Lumasphere;
        use StateChange::*;
        map.add_unipolar(GROUP, "lamp_1_intensity", |v| {
            Lumasphere(Lamp1Intensity(unipolar_fader_with_detent(v)))
        });
        map.add_unipolar(GROUP, "lamp_2_intensity", |v| {
            Lumasphere(Lamp2Intensity(unipolar_fader_with_detent(v)))
        });

        map.add_bipolar(GROUP, "ball_rotation", |v| {
            Lumasphere(BallRotation(bipolar_fader_with_detent(v)))
        });
        BALL_START.map_state(map, |v| Lumasphere(BallStart(v)));

        map.add_unipolar(GROUP, "color_rotation", |v| {
            Lumasphere(ColorRotation(unipolar_fader_with_detent(v)))
        });
        COLOR_START.map_state(map, |v| Lumasphere(ColorStart(v)));
        map_strobe(map, 1, |inner| Lumasphere(Strobe1(inner)));
        map_strobe(map, 2, |inner| Lumasphere(Strobe2(inner)));
    }
}

fn map_strobe<W>(map: &mut ControlMap<FixtureControlMessage>, index: u8, wrap: W)
where
    W: Fn(StrobeStateChange) -> FixtureControlMessage + 'static + Copy,
{
    use GenericStrobeStateChange::*;
    use StrobeStateChange::*;
    map.add_bool(GROUP, &format!("strobe_{}_state", index), move |v| {
        wrap(State(On(v)))
    });
    map.add_unipolar(GROUP, &format!("strobe_{}_rate", index), move |v| {
        wrap(State(Rate(v)))
    });
    map.add_unipolar(GROUP, &format!("strobe_{}_intensity", index), move |v| {
        wrap(Intensity(v))
    });
}

impl HandleStateChange<StateChange> for Lumasphere {}
