use rosc::OscMessage;

use super::ControlMap;
use crate::fixture::ControlMessage::{self as ShowControlMessage, Lumasphere};
use crate::generic::GenericStrobeStateChange;
use crate::lumasphere::StateChange;
use crate::lumasphere::StrobeStateChange;
use crate::util::bipolar_fader_with_detent;
use crate::util::unipolar_fader_with_detent;

const GROUP: &str = "Lumasphere";

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
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
    map.add_bool(GROUP, "ball_start", |v| Lumasphere(BallStart(v)));

    map.add_unipolar(GROUP, "color_rotation", |v| {
        Lumasphere(ColorRotation(unipolar_fader_with_detent(v)))
    });
    map.add_bool(GROUP, "color_start", |v| Lumasphere(ColorStart(v)));
    map_strobe(map, 1, |inner| Lumasphere(Strobe1(inner)));
    map_strobe(map, 2, |inner| Lumasphere(Strobe2(inner)));
}

fn map_strobe<W>(map: &mut ControlMap<ShowControlMessage>, index: u8, wrap: W)
where
    W: Fn(StrobeStateChange) -> ShowControlMessage + 'static + Copy,
{
    use GenericStrobeStateChange::*;
    use StrobeStateChange::*;
    map.add_bool(GROUP, format!("strobe_{}_state", index), move |v| {
        wrap(State(On(v)))
    });
    map.add_unipolar(GROUP, format!("strobe_{}_rate", index), move |v| {
        wrap(State(Rate(v)))
    });
    map.add_unipolar(GROUP, format!("strobe_{}_intensity", index), move |v| {
        wrap(Intensity(v))
    });
}

pub fn handle_state_change<S>(_: StateChange, _: &mut S)
where
    S: FnMut(OscMessage),
{
    // No lumasphere OSC controls use talkback.
}
