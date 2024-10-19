//! Martin Rush-series Wizard (still not as good as the OG).

use log::error;
use number::{BipolarFloat, UnipolarFloat};

use super::generic::{GenericStrobe, GenericStrobeStateChange};
use crate::fixture::prelude::*;
use crate::osc::prelude::*;
use crate::util::{bipolar_to_range, bipolar_to_split_range, unipolar_to_range};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Default, Debug)]
pub struct RushWizard {
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
}

impl PatchFixture for RushWizard {
    const NAME: FixtureType = FixtureType("RushWizard");
    fn channel_count(&self) -> usize {
        10
    }
}

impl RushWizard {
    const GOBO_COUNT: usize = 16; // includes the open position

    fn handle_state_change(&mut self, sc: StateChange, emitter: &FixtureStateEmitter) {
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
        Self::emit(sc, emitter);
    }
}

impl NonAnimatedFixture for RushWizard {
    fn render(&self, group_controls: &FixtureGroupControls, dmx_buf: &mut [u8]) {
        dmx_buf[0] = self
            .strobe
            .render_range_with_master(group_controls.strobe(), 8, 16, 131);

        dmx_buf[1] = unipolar_to_range(0, 255, self.dimmer);
        dmx_buf[2] = if self.twinkle {
            // WHY did you put twinkle on the color wheel...
            unipolar_to_range(221, 243, self.twinkle_speed)
        } else {
            self.color.as_dmx()
        };
        dmx_buf[3] = (self.gobo as u8) * 2 + 160;
        dmx_buf[4] = bipolar_to_split_range(self.drum_rotation, 190, 128, 193, 255, 191);
        dmx_buf[5] = bipolar_to_range(0, 120, self.drum_swivel);
        dmx_buf[6] = bipolar_to_split_range(self.reflector_rotation, 190, 128, 193, 255, 191);
        dmx_buf[7] = 0;
        dmx_buf[8] = 0;
        dmx_buf[9] = 0;
    }
}

impl ControllableFixture for RushWizard {
    fn populate_controls(&mut self) {
        Self::map_controls(&mut self.controls);
    }

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        Self::emit(Dimmer(self.dimmer), emitter);
        let mut emit_strobe = |ssc| {
            Self::emit(Strobe(ssc), emitter);
        };
        self.strobe.emit_state(&mut emit_strobe);
        Self::emit(Color(self.color), emitter);
        Self::emit(Twinkle(self.twinkle), emitter);
        Self::emit(TwinkleSpeed(self.twinkle_speed), emitter);
        Self::emit(Gobo(self.gobo), emitter);
        Self::emit(DrumRotation(self.drum_rotation), emitter);
        Self::emit(DrumSwivel(self.drum_swivel), emitter);
        Self::emit(ReflectorRotation(self.reflector_rotation), emitter);
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
    Magenta,
    Yellow,
    DarkBlue,
    White,
    Red,
    Orange,
    Green,
}

impl Color {
    fn as_dmx(self) -> u8 {
        use Color::*;
        match self {
            Open => 159,
            Blue => 161,
            Magenta => 164,
            Yellow => 167,
            DarkBlue => 170,
            White => 173,
            Red => 176,
            Orange => 179,
            Green => 182,
        }
    }
}

const GROUP: &str = RushWizard::NAME.0;
const COLOR: &str = "Color";

const GOBO_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Gobo",
    n: 16,
    x_primary_coordinate: false,
};

const TWINKLE: Button = button(GROUP, "Twinkle");

impl EnumRadioButton for Color {}

impl RushWizard {
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
        map.add_bipolar("DrumSwivel", DrumSwivel);
        map.add_bipolar("ReflectorRotation", |v| {
            ReflectorRotation(bipolar_fader_with_detent(v))
        });
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessage {
    StateChange::Strobe(sc)
}

impl HandleOscStateChange<StateChange> for RushWizard {
    fn emit_osc_state_change<S>(sc: StateChange, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        match sc {
            StateChange::Color(c) => {
                c.set(GROUP, COLOR, send);
            }
            StateChange::Gobo(v) => GOBO_SELECT.set(v, send),
            _ => (),
        }
    }
}
