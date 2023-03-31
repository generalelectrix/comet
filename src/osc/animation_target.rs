use log::error;
use rosc::OscMessage;
use tunnels::clock_bank::{ClockIdxExt, N_CLOCKS};

use crate::fixture::animation_target::AnimationTarget;
use crate::fixture::wizard_extreme::AnimationTarget as WizardExtremeAnimationTarget;
use crate::fixture::{EmitStateChange, FixtureControlMessage, N_ANIM};
use crate::osc::radio_button::EnumRadioButton;
use crate::osc::{ignore_payload, HandleStateChange};
use crate::osc::{send_button, send_float, ControlMap, MapControls, RadioButton};
use crate::util::bipolar_fader_with_detent;
use tunnels::animation::{ControlMessage, StateChange, Waveform::*};

const GROUP: &str = "Animation";
const TARGET: &str = "Target";

const ANIMATION_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Select",
    n: N_ANIM,
    x_primary_coordinate: false,
};

impl EnumRadioButton for WizardExtremeAnimationTarget {}
pub struct AnimationTargetControls;

impl MapControls for AnimationTargetControls {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use ControlMessage::*;
        use FixtureControlMessage::{AnimationSelect, AnimationTarget};

        map.add_enum_handler(GROUP, TARGET, ignore_payload, |t, _| {
            AnimationTarget(crate::fixture::animation_target::AnimationTarget::WizardExtreme(t))
        });
        map.add_radio_button_array(ANIMATION_SELECT, |v| AnimationSelect(v));
    }
}

impl HandleStateChange<AnimationTarget> for AnimationTargetControls {
    fn emit_state_change<S>(sc: AnimationTarget, send: &mut S)
    where
        S: FnMut(OscMessage),
    {
        match sc {
            AnimationTarget::WizardExtreme(t) => t.set(GROUP, TARGET, send),
            AnimationTarget::None => (),
        }
    }
}

impl HandleStateChange<usize> for AnimationTargetControls {
    fn emit_state_change<S>(sc: usize, send: &mut S)
    where
        S: FnMut(OscMessage),
    {
        ANIMATION_SELECT.set(sc, send);
    }
}
