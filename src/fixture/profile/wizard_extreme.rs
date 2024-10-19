//! Martin Wizard Extreme - the one that Goes Slow

use log::error;
use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Debug)]
struct Active(bool);

impl Default for Active {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Default, Debug)]
pub struct WizardExtreme {
    controls: GroupControlMap<ControlMessage>,
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
    const NAME: FixtureType = FixtureType("WizardExtreme");
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
    fn populate_controls(&mut self) {
        Self::map_controls(&mut self.controls);
    }

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
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        let Some((ctl, _)) = self.controls.handle(msg)? else {
            return Ok(());
        };
        self.handle_state_change(ctl, emitter);
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


const COLOR: &str = "Color";

const GOBO_SELECT: RadioButton = RadioButton {
    control: "Gobo",
    n: WizardExtreme::GOBO_COUNT,
    x_primary_coordinate: false,
};

const TWINKLE: Button = button("Twinkle");

const MIRROR_DRUM_ROTATION: Button = button("MirrorDrumRotation");
const MIRROR_DRUM_SWIVEL: Button = button("MirrorDrumSwivel");
const MIRROR_REFLECTOR_ROTATION: Button = button("MirrorReflectorRotation");

const ACTIVE: Button = button("Active");

impl EnumRadioButton for Color {}

impl WizardExtreme {
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;
        map.add_unipolar("Dimmer", Dimmer);
        map_strobe(map, "Strobe", &wrap_strobe);
        map.add_enum_handler(COLOR, ignore_payload, |c, _| Color(c));
        TWINKLE.map_state(map, Twinkle);
        map.add_unipolar("TwinkleSpeed", TwinkleSpeed);
        GOBO_SELECT.map(map, Gobo);
        map.add_bipolar("DrumRotation", |v| {
            DrumRotation(bipolar_fader_with_detent(v))
        });
        MIRROR_DRUM_ROTATION.map_state(map, MirrorDrumRotation);
        map.add_bipolar("DrumSwivel", DrumSwivel);
        MIRROR_DRUM_SWIVEL.map_state(map, MirrorDrumSwivel);
        map.add_bipolar("ReflectorRotation", |v| {
            ReflectorRotation(bipolar_fader_with_detent(v))
        });
        MIRROR_REFLECTOR_ROTATION.map_state(map, MirrorReflectorRotation);
        ACTIVE.map_state(map, Active);
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessage {
    StateChange::Strobe(sc)
}

impl HandleOscStateChange<StateChange> for WizardExtreme {
    fn emit_osc_state_change<S>(sc: StateChange, send: &S)
    where
        S: crate::osc::EmitScopedOscMessage + ?Sized,
    {
        match sc {
            StateChange::Dimmer(v) => {
                send_float("Dimmer", v, send);
            }
            StateChange::Color(c) => {
                c.set(COLOR, send);
            }
            StateChange::Gobo(v) => GOBO_SELECT.set(v, send),
            StateChange::MirrorDrumRotation(v) => MIRROR_DRUM_ROTATION.send(v, send),
            StateChange::MirrorReflectorRotation(v) => MIRROR_REFLECTOR_ROTATION.send(v, send),
            StateChange::MirrorDrumSwivel(v) => MIRROR_DRUM_SWIVEL.send(v, send),
            StateChange::Active(v) => ACTIVE.send(v, send),
            _ => (),
        }
    }
}
