//! Martin Wizard Extreme - the one that Goes Slow

use anyhow::Context;
use log::{debug, error};
use num_derive::{FromPrimitive, ToPrimitive};
use number::{BipolarFloat, UnipolarFloat};

use super::generic::{GenericStrobe, GenericStrobeStateChange};
use crate::channel::{ChannelControlMessage, ChannelStateChange};
use crate::fixture::prelude::*;
use crate::util::{bipolar_to_range, bipolar_to_split_range, unipolar_to_range};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Debug)]
struct Active(bool);

impl Default for Active {
    fn default() -> Self {
        Self(true)
    }
}

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
    mirror: Mirror,
    active: Active,
}

impl PatchAnimatedFixture for WizardExtreme {
    const NAME: FixtureType = FixtureType("wizard_extreme");
    fn channel_count(&self) -> usize {
        11
    }
}

impl WizardExtreme {
    pub const GOBO_COUNT: usize = 14; // includes the open position

    fn handle_state_change(&mut self, sc: StateChange, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        match sc {
            Dimmer(v) => {
                self.dimmer = v;
                emitter.emit_channel(ChannelStateChange::Level(v));
            }
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
            MirrorDrumRotation(v) => self.mirror.drum_rotation = v,
            DrumSwivel(v) => self.drum_swivel = v,
            MirrorDrumSwivel(v) => self.mirror.drum_swivel = v,
            ReflectorRotation(v) => self.reflector_rotation = v,
            MirrorReflectorRotation(v) => self.mirror.reflector_rotation = v,
            Active(v) => self.active.0 = v,
        };
        Self::emit(sc, emitter);
    }
}

impl ControllableFixture for WizardExtreme {
    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        Self::emit(Dimmer(self.dimmer), emitter);
        emitter.emit_channel(crate::channel::ChannelStateChange::Level(self.dimmer));

        let mut emit_strobe = |ssc| {
            Self::emit(Strobe(ssc), emitter);
        };
        self.strobe.emit_state(&mut emit_strobe);
        Self::emit(Color(self.color), emitter);
        Self::emit(Twinkle(self.twinkle), emitter);
        Self::emit(TwinkleSpeed(self.twinkle_speed), emitter);
        Self::emit(Gobo(self.gobo), emitter);
        Self::emit(DrumRotation(self.drum_rotation), emitter);
        Self::emit(MirrorDrumRotation(self.mirror.drum_rotation), emitter);
        Self::emit(DrumSwivel(self.drum_swivel), emitter);
        Self::emit(MirrorDrumSwivel(self.mirror.drum_swivel), emitter);
        Self::emit(ReflectorRotation(self.reflector_rotation), emitter);
        Self::emit(
            MirrorReflectorRotation(self.mirror.reflector_rotation),
            emitter,
        );
        Self::emit(Active(self.active.0), emitter);
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        self.handle_state_change(
            *msg.unpack_as::<ControlMessage>().context(Self::NAME)?,
            emitter,
        );
        Ok(())
    }

    fn control_from_channel(&mut self, msg: &ChannelControlMessage, emitter: &FixtureStateEmitter) {
        match msg {
            ChannelControlMessage::Level(l) => {
                self.handle_state_change(StateChange::Dimmer(*l), emitter);
            }
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
        if !self.active.0 {
            dmx_buf.fill(0);
            return;
        }
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
        dmx_buf[1] = bipolar_to_split_range(
            BipolarFloat::new(reflector_rotation)
                .invert_if(group_controls.mirror && self.mirror.reflector_rotation),
            2,
            63,
            127,
            66,
            0,
        );

        dmx_buf[2] = if self.twinkle {
            // WHY did you put twinkle on the color wheel...
            unipolar_to_range(176, 243, UnipolarFloat::new(twinkle_speed))
        } else {
            self.color.as_dmx()
        };
        dmx_buf[3] = 0; // color shake
        dmx_buf[4] = (self.gobo as u8) * 12;
        dmx_buf[5] = 0; // gobo shake
        dmx_buf[6] = bipolar_to_range(
            0,
            127,
            BipolarFloat::new(drum_swivel)
                .invert_if(group_controls.mirror && self.mirror.drum_swivel),
        );
        dmx_buf[7] = bipolar_to_split_range(
            BipolarFloat::new(drum_rotation)
                .invert_if(group_controls.mirror && self.mirror.drum_rotation),
            2,
            63,
            127,
            66,
            0,
        );
        dmx_buf[8] = 0;
        dmx_buf[9] = 0;
        dmx_buf[10] = 0;
    }
}

#[derive(Debug)]
struct Mirror {
    drum_rotation: bool,
    drum_swivel: bool,
    reflector_rotation: bool,
}

impl Default for Mirror {
    fn default() -> Self {
        Self {
            drum_rotation: true,
            drum_swivel: true,
            reflector_rotation: true,
        }
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
    MirrorReflectorRotation(bool),
    MirrorDrumRotation(bool),
    MirrorDrumSwivel(bool),
    Active(bool),
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
