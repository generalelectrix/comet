//! Martin Rush-series Wizard (still not as good as the OG).

use log::error;
use number::{BipolarFloat, UnipolarFloat};

use super::generic::{GenericStrobe, GenericStrobeStateChange};
use super::{EmitFixtureStateChange, Fixture, FixtureControlMessage, PatchFixture};
use crate::util::{bipolar_to_range, bipolar_to_split_range, unipolar_to_range};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Default, Debug)]
pub struct RushWizard {
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
    const CHANNEL_COUNT: usize = 10;
}

impl RushWizard {
    const GOBO_COUNT: usize = 16; // includes the open position

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
        emitter.emit_rush_wizard(sc);
    }
}

impl Fixture for RushWizard {
    fn render(&self, dmx_buf: &mut [u8]) {
        dmx_buf[0] = if self.strobe.on() {
            unipolar_to_range(16, 131, self.strobe.rate())
        } else {
            8
        };
        dmx_buf[1] = unipolar_to_range(0, 255, self.dimmer);
        dmx_buf[2] = if self.twinkle {
            // WHY did you put twinkle on the color wheel...
            unipolar_to_range(221, 243, self.twinkle_speed)
        } else {
            self.color.as_dmx()
        };
        dmx_buf[3] = (self.gobo as u8) * 2 + 160;
        dmx_buf[4] = bipolar_to_split_range(self.drum_rotation, 128, 190, 193, 255, 191);
        dmx_buf[5] = bipolar_to_range(0, 120, self.drum_swivel);
        dmx_buf[6] = bipolar_to_split_range(self.reflector_rotation, 128, 190, 193, 255, 191);
        dmx_buf[7] = 0;
        dmx_buf[8] = 0;
        dmx_buf[9] = 0;
    }

    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        emitter.emit_rush_wizard(Dimmer(self.dimmer));
        let mut emit_strobe = |ssc| {
            emitter.emit_rush_wizard(Strobe(ssc));
        };
        self.strobe.emit_state(&mut emit_strobe);
        emitter.emit_rush_wizard(Color(self.color));
        emitter.emit_rush_wizard(Twinkle(self.twinkle));
        emitter.emit_rush_wizard(TwinkleSpeed(self.twinkle_speed));
        emitter.emit_rush_wizard(Gobo(self.gobo));
        emitter.emit_rush_wizard(DrumRotation(self.drum_rotation));
        emitter.emit_rush_wizard(DrumSwivel(self.drum_swivel));
        emitter.emit_rush_wizard(ReflectorRotation(self.reflector_rotation));
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::RushWizard(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
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
