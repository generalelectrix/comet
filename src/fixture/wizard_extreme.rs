//! Martin Wizard Extreme - the one that Goes Slow

use log::{debug, error};
use num_derive::{FromPrimitive, ToPrimitive};
use number::{BipolarFloat, UnipolarFloat};

use super::animation_target::TargetedAnimationValues;
use super::generic::{GenericStrobe, GenericStrobeStateChange};
use super::{
    AnimatedFixture, ControllableFixture, EmitFixtureStateChange, FixtureControlMessage,
    PatchAnimatedFixture,
};
use crate::master::FixtureGroupControls;
use crate::util::{bipolar_to_range, bipolar_to_split_range, unipolar_to_range};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Default, Debug)]
pub struct WizardExtreme {
    dimmer: UnipolarFloat,
    strobe: GenericStrobe,
    color: Color,
    twinkle: bool,
    twinkle_speed: UnipolarFloat,
    gobo: usize,
    drum_rotation: BipolarFloat,
    drum_swivel: BipolarFloat,
    reflector_rotation: BipolarFloat,
}

impl PatchAnimatedFixture for WizardExtreme {
    const NAME: &'static str = "wizard_extreme";
    fn channel_count(&self) -> usize {
        11
    }
}

impl WizardExtreme {
    const GOBO_COUNT: usize = 14; // includes the open position

    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        match sc {
            Dimmer(v) => self.dimmer = v,
            Strobe(sc) => self.strobe.handle_state_change(sc),
            Color(c) => self.color = c,
            Twinkle(v) => self.twinkle = v,
            TwinkleSpeed(v) => self.twinkle_speed = v,
            Gobo(v) => {
                if v >= Self::GOBO_COUNT {
                    error!("Gobo select index {} out of range.", v);
                    return;
                }
                self.gobo = v;
            }
            DrumRotation(v) => self.drum_rotation = v,
            DrumSwivel(v) => self.drum_swivel = v,
            ReflectorRotation(v) => self.reflector_rotation = v,
        };
        emitter.emit_wizard_extreme(sc);
    }
}

impl ControllableFixture for WizardExtreme {
    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        emitter.emit_wizard_extreme(Dimmer(self.dimmer));
        let mut emit_strobe = |ssc| {
            emitter.emit_wizard_extreme(Strobe(ssc));
        };
        self.strobe.emit_state(&mut emit_strobe);
        emitter.emit_wizard_extreme(Color(self.color));
        emitter.emit_wizard_extreme(Twinkle(self.twinkle));
        emitter.emit_wizard_extreme(TwinkleSpeed(self.twinkle_speed));
        emitter.emit_wizard_extreme(Gobo(self.gobo));
        emitter.emit_wizard_extreme(DrumRotation(self.drum_rotation));
        emitter.emit_wizard_extreme(DrumSwivel(self.drum_swivel));
        emitter.emit_wizard_extreme(ReflectorRotation(self.reflector_rotation));
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::WizardExtreme(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }
}

impl AnimatedFixture for WizardExtreme {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: &TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        debug!("{:?}", animation_vals);
        let mut drum_swivel = self.drum_swivel.val();
        let mut drum_rotation = self.drum_rotation.val();
        let mut reflector_rotation = self.reflector_rotation.val();
        let mut dimmer = self.dimmer.val();
        let mut twinkle_speed = self.twinkle_speed.val();
        for (val, target) in animation_vals {
            use AnimationTarget::*;
            match target {
                DrumSwivel => drum_swivel += val,
                DrumRotation => drum_rotation += val,
                ReflectorRotation => reflector_rotation += val,
                // FIXME: might want to do something nicer for unipolar values
                Dimmer => dimmer += val,
                TwinkleSpeed => twinkle_speed += val,
            }
        }
        dmx_buf[0] = {
            let strobe_off = 0;
            let strobe =
                self.strobe
                    .render_range_with_master(group_controls.strobe(), strobe_off, 189, 130);
            if strobe == strobe_off {
                unipolar_to_range(0, 129, UnipolarFloat::new(dimmer))
            } else {
                strobe
            }
        };
        dmx_buf[1] =
            bipolar_to_split_range(BipolarFloat::new(reflector_rotation), 2, 63, 127, 66, 0);

        dmx_buf[2] = if self.twinkle {
            // WHY did you put twinkle on the color wheel...
            unipolar_to_range(176, 243, UnipolarFloat::new(twinkle_speed))
        } else {
            self.color.as_dmx()
        };
        dmx_buf[3] = 0; // color shake
        dmx_buf[4] = (self.gobo as u8) * 12;
        dmx_buf[5] = 0; // gobo shake
        dmx_buf[6] = bipolar_to_range(0, 127, BipolarFloat::new(drum_swivel));
        dmx_buf[7] = bipolar_to_split_range(BipolarFloat::new(drum_rotation), 2, 63, 127, 66, 0);
        dmx_buf[8] = 0;
        dmx_buf[9] = 0;
        dmx_buf[10] = 0;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    Dimmer(UnipolarFloat),
    Strobe(GenericStrobeStateChange),
    Color(Color),
    Twinkle(bool),
    TwinkleSpeed(UnipolarFloat),
    Gobo(usize),
    DrumRotation(BipolarFloat),
    DrumSwivel(BipolarFloat),
    ReflectorRotation(BipolarFloat),
}

pub type ControlMessage = StateChange;

#[derive(Copy, Clone, Debug, Default, PartialEq, EnumString, EnumIter, EnumDisplay)]
pub enum Color {
    #[default]
    Open,
    Blue,
    Orange,
    Purple,
    Green,
    DarkBlue,
    Yellow,
    Magenta,
}

impl Color {
    fn as_dmx(self) -> u8 {
        use Color::*;
        match self {
            Open => 0,
            Blue => 12,
            Orange => 24,
            Purple => 36,
            Green => 48,
            DarkBlue => 60,
            Yellow => 72,
            Magenta => 84,
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    EnumString,
    EnumIter,
    EnumDisplay,
    FromPrimitive,
    ToPrimitive,
)]
pub enum AnimationTarget {
    #[default]
    Dimmer,
    TwinkleSpeed,
    DrumRotation,
    DrumSwivel,
    ReflectorRotation,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        matches!(self, Self::Dimmer | Self::TwinkleSpeed)
    }
}
