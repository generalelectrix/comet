use super::ControlMap;
use crate::lumasphere::StateChange;
use crate::lumasphere::StrobeStateChange;
use crate::util::bipolar_fader_with_detent;
use crate::util::unipolar_fader_with_detent;
use crate::{lumasphere::ControlMessage, osc::quadratic};

const GROUP: &str = "Lumasphere";

pub fn map_lumasphere_controls(map: &mut ControlMap<ControlMessage>) {
    use StateChange::*;
    map.add_unipolar(GROUP, "lamp_1_intensity", |v| {
        Lamp1Intensity(unipolar_fader_with_detent(v))
    });
    map.add_unipolar(GROUP, "lamp_2_intensity", |v| {
        Lamp2Intensity(unipolar_fader_with_detent(v))
    });

    map.add_bipolar(GROUP, "ball_rotation", |v| {
        BallRotation(bipolar_fader_with_detent(v))
    });
    map.add_bool(GROUP, "ball_start", BallStart);

    map.add_unipolar(GROUP, "color_rotation", |v| {
        ColorRotation(unipolar_fader_with_detent(v))
    });
    map.add_bool(GROUP, "color_start", ColorStart);
    map_strobe(map, 1, Strobe1);
    map_strobe(map, 2, Strobe2);
}

fn map_strobe<W>(map: &mut ControlMap<ControlMessage>, index: u8, wrap: W)
where
    W: Fn(StrobeStateChange) -> StateChange + 'static + Copy,
{
    use StrobeStateChange::*;
    map.add_bool(GROUP, format!("strobe_{}_state", index), move |v| {
        wrap(On(v))
    });
    map.add_unipolar(GROUP, format!("strobe_{}_intensity", index), move |v| {
        wrap(Intensity(v))
    });
    map.add_unipolar(GROUP, format!("strobe_{}_rate", index), move |v| {
        wrap(Rate(v))
    });
}
