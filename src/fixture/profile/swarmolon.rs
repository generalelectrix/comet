//! Control profle for the Chauvet Swarm 5 FX, aka the Swarmolon.
//! Also
use anyhow::{Context, Result};
use log::error;
use number::{BipolarFloat, UnipolarFloat};

use super::generic::{GenericStrobe, GenericStrobeStateChange};
use crate::fixture::prelude::*;
use crate::util::{bipolar_to_split_range, unipolar_to_range};
use strum::IntoEnumIterator;
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Default, Debug)]
pub struct Swarmolon {
    derby_color: DerbyColorState,
    derby_strobe: GenericStrobe,
    derby_rotation: BipolarFloat,
    white_strobe: WhiteStrobe,
    red_laser_on: bool,
    green_laser_on: bool,
    laser_strobe: GenericStrobe,
    laser_rotation: BipolarFloat,
    /// If true, duplicate the derby settings to a slaved quad phase.
    /// The quad phase is assumed to be addressed just after the Swarmolon.
    quad_phase_mindmeld: bool,
    /// If true, duplicate the laser settings to a slaved Galaxian 3D.
    /// The galaxian is assumed to be addressed after the also-slaved quad phase.
    galaxian_mindmeld: bool,
}

const CHANNEL_COUNT: usize = 9;
const QUAD_PHASE_CHANNEL_COUNT: usize = 4;
const GALAXIAN_CHANNEL_COUNT: usize = 5;

impl PatchFixture for Swarmolon {
    const NAME: FixtureType = FixtureType("swarmolon");
    fn channel_count(&self) -> usize {
        let mut count = CHANNEL_COUNT;
        if self.quad_phase_mindmeld {
            count += QUAD_PHASE_CHANNEL_COUNT;
        }
        if self.galaxian_mindmeld {
            count += GALAXIAN_CHANNEL_COUNT;
        }
        count
    }

    fn new(options: &std::collections::HashMap<String, String>) -> Result<Self> {
        let mut s = Self::default();
        if options.contains_key("quad_phase") {
            s.quad_phase_mindmeld = true;
        }
        if options.contains_key("galaxian") {
            s.galaxian_mindmeld = true;
        }
        Ok(s)
    }
}

impl Swarmolon {
    fn handle_state_change(
        &mut self,
        sc: StateChange,
        emitter: &mut dyn crate::osc::EmitControlMessage,
    ) {
        use StateChange::*;
        match sc {
            DerbyColor(color, state) => {
                self.derby_color.set(color, state);
            }
            DerbyStrobe(sc) => self.derby_strobe.handle_state_change(sc),
            DerbyRotation(v) => self.derby_rotation = v,
            WhiteStrobe(sc) => {
                if let Err(e) = self.white_strobe.handle_state_change(sc) {
                    error!("{}", e);
                    return;
                }
            }
            RedLaserOn(v) => self.red_laser_on = v,
            GreenLaserOn(v) => self.green_laser_on = v,
            LaserStrobe(sc) => self.laser_strobe.handle_state_change(sc),
            LaserRotation(v) => self.laser_rotation = v,
        };
        Self::emit(sc, emitter);
    }
}

impl NonAnimatedFixture for Swarmolon {
    fn render(&self, group_controls: &FixtureGroupControls, dmx_buf: &mut [u8]) {
        dmx_buf[0] = 255; // always set to DMX mode
        dmx_buf[1] = self.derby_color.render();
        dmx_buf[2] = 0; // Not using automatic derby programs.
        dmx_buf[3] =
            self.derby_strobe
                .render_range_with_master(group_controls.strobe(), 0, 254, 10);
        dmx_buf[4] = self.white_strobe.render(group_controls);
        dmx_buf[5] = match (self.red_laser_on, self.green_laser_on) {
            (false, false) => 0,
            (true, false) => 10,
            (false, true) => 50,
            (true, true) => 255, // TODO: verify this is actually correct.
        };
        dmx_buf[6] = self
            .laser_strobe
            .render_range_with_master(group_controls.strobe(), 0, 5, 254);
        dmx_buf[7] = bipolar_to_split_range(self.derby_rotation, 5, 127, 134, 255, 0);
        dmx_buf[8] = bipolar_to_split_range(self.laser_rotation, 5, 127, 134, 255, 0);
        let mut offset = CHANNEL_COUNT;
        if self.quad_phase_mindmeld {
            let slice = &mut dmx_buf[offset..offset + QUAD_PHASE_CHANNEL_COUNT];
            offset += QUAD_PHASE_CHANNEL_COUNT;
            let color_val = self.derby_color.render_quad_phase();
            slice[0] = color_val;
            slice[1] = bipolar_to_split_range(
                squash_quad_phase_rotation(self.derby_rotation),
                120,
                10,
                135,
                245,
                0,
            );
            slice[2] =
                self.derby_strobe
                    .render_range_with_master(group_controls.strobe(), 0, 1, 255);
            slice[3] = if color_val == 0 { 0 } else { 255 };
        }
        if self.galaxian_mindmeld {
            let slice = &mut dmx_buf[offset..offset + GALAXIAN_CHANNEL_COUNT];
            slice[0] = if !self.red_laser_on {
                0
            // FIXME: this won't work if we don't use master strobe in the future.
            } else if !group_controls.strobe().state.on() {
                8
            } else {
                self.laser_strobe
                    .render_range_with_master(group_controls.strobe(), 8, 16, 239)
            };
            slice[1] = if !self.green_laser_on {
                0
            // FIXME: this won't work if we don't use master strobe in the future.
            } else if !group_controls.strobe().state.on() {
                8
            } else {
                self.laser_strobe
                    .render_range_with_master(group_controls.strobe(), 8, 16, 239)
            };
            let laser_rotation =
                bipolar_to_split_range(self.laser_rotation, 194, 255, 189, 128, 190);
            slice[2] = laser_rotation;
            slice[3] = laser_rotation;
            slice[4] = 0;
        }
    }
}

impl ControllableFixture for Swarmolon {
    fn emit_state(&self, emitter: &mut dyn crate::osc::EmitControlMessage) {
        use StateChange::*;
        self.derby_color.emit_state(emitter);
        let mut emit_derby_strobe = |ssc| {
            Self::emit(DerbyStrobe(ssc), emitter);
        };
        self.derby_strobe.emit_state(&mut emit_derby_strobe);
        Self::emit(DerbyRotation(self.derby_rotation), emitter);
        let mut emit_white_strobe = |ssc| {
            Self::emit(WhiteStrobe(ssc), emitter);
        };
        self.white_strobe.emit_state(&mut emit_white_strobe);
        Self::emit(RedLaserOn(self.red_laser_on), emitter);
        Self::emit(GreenLaserOn(self.green_laser_on), emitter);
        let mut emit_laser_strobe = |ssc| {
            Self::emit(LaserStrobe(ssc), emitter);
        };
        self.laser_strobe.emit_state(&mut emit_laser_strobe);
        Self::emit(LaserRotation(self.laser_rotation), emitter);
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn crate::osc::EmitControlMessage,
    ) -> anyhow::Result<()> {
        match *msg.unpack_as::<ControlMessage>().context(Self::NAME)? {
            ControlMessage::Set(sc) => {
                self.handle_state_change(sc, emitter);
            }
            ControlMessage::StrobeRate(v) => {
                self.handle_state_change(
                    StateChange::DerbyStrobe(GenericStrobeStateChange::Rate(v)),
                    emitter,
                );
                self.handle_state_change(
                    StateChange::WhiteStrobe(WhiteStrobeStateChange::State(
                        GenericStrobeStateChange::Rate(v),
                    )),
                    emitter,
                );
                self.handle_state_change(
                    StateChange::LaserStrobe(GenericStrobeStateChange::Rate(v)),
                    emitter,
                );
            }
        }

        Ok(())
    }
}

/// The swarmolon has incredibly weird rotation speed.
/// Squash most of the quad phase speed range into the top of the fader to try
/// to match them.
pub fn squash_quad_phase_rotation(v: BipolarFloat) -> BipolarFloat {
    BipolarFloat::new(v.val().powi(5).copysign(v.val()))
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    DerbyColor(DerbyColor, bool),
    DerbyStrobe(GenericStrobeStateChange),
    DerbyRotation(BipolarFloat),
    WhiteStrobe(WhiteStrobeStateChange),
    RedLaserOn(bool),
    GreenLaserOn(bool),
    LaserStrobe(GenericStrobeStateChange),
    LaserRotation(BipolarFloat),
}

#[derive(Clone, Copy, Debug)]
pub enum ControlMessage {
    Set(StateChange),
    /// Command to set the state of all of the fixture's strobe rates.
    StrobeRate(UnipolarFloat),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumString, EnumIter, EnumDisplay, PartialOrd, Ord)]
pub enum DerbyColor {
    Red,
    Green,
    Blue,
    Amber,
    White,
}

#[derive(Debug)]
struct DerbyColorState(Vec<DerbyColor>);

impl Default for DerbyColorState {
    fn default() -> Self {
        Self(Vec::with_capacity(5))
    }
}

impl DerbyColorState {
    pub fn set(&mut self, color: DerbyColor, add: bool) {
        if !add {
            self.0.retain(|v| *v != color);
            return;
        }
        if self.0.contains(&color) {
            return;
        }
        self.0.push(color);
        self.0.sort();
    }

    pub fn emit_state(&self, emitter: &mut dyn crate::osc::EmitControlMessage) {
        for color in DerbyColor::iter() {
            let state = self.0.contains(&color);
            Swarmolon::emit(StateChange::DerbyColor(color, state), emitter);
        }
    }

    pub fn render(&self) -> u8 {
        use DerbyColor::*;
        match self.0[..] {
            [] => 0,
            [Red] => 10,
            [Green] => 15,
            [Blue] => 20,
            [Amber] => 25,
            [White] => 30,
            [Red, White] => 35,
            [Red, Green] => 40,
            [Green, Blue] => 45,
            [Blue, Amber] => 50,
            [Amber, White] => 55,
            [Green, White] => 60,
            [Green, Amber] => 65,
            [Red, Amber] => 70,
            [Red, Blue] => 75,
            [Blue, White] => 80,
            [Red, Green, Blue] => 85,
            [Red, Green, Amber] => 90,
            [Red, Green, White] => 95,
            [Red, Blue, Amber] => 100,
            [Red, Blue, White] => 105,
            [Red, Amber, White] => 110,
            [Green, Blue, Amber] => 115,
            [Green, Blue, White] => 120,
            [Green, Amber, White] => 125,
            [Blue, Amber, White] => 130,
            [Red, Green, Blue, Amber] => 135,
            [Red, Green, Blue, White] => 140,
            [Green, Blue, Amber, White] => 145,
            [Red, Green, Amber, White] => 150,
            [Red, Blue, Amber, White] => 155,
            [Red, Green, Blue, Amber, White] => 160,
            _ => {
                error!("Unmatched derby color state: {:?}.", self.0);
                0
            }
        }
    }

    /// Return the DMX color setting.
    /// If 0, we should also close the shutter.
    pub fn render_quad_phase(&self) -> u8 {
        use DerbyColor::*;
        match self.0[..] {
            [] | [Amber] => 0,
            [Red] | [Red, Amber] => 1,
            [Green] | [Green, Amber] => 17,
            [Blue] | [Blue, Amber] => 34,
            [White] | [Amber, White] => 51,
            [Red, Green] | [Red, Green, Amber] => 68,
            [Red, Blue] | [Red, Blue, Amber] => 85,
            [Red, White] | [Red, Amber, White] => 102,
            [Green, Blue] | [Green, Blue, Amber] => 119,
            [Green, White] | [Green, Amber, White] => 136,
            [Blue, White] | [Blue, Amber, White] => 153,
            [Red, Green, Blue] | [Red, Green, Blue, Amber] => 170,
            [Red, Green, White] | [Red, Green, Amber, White] => 187,
            [Red, Blue, White] | [Red, Blue, Amber, White] => 204,
            [Green, Blue, White] | [Green, Blue, Amber, White] => 221,
            [Red, Green, Blue, White] | [Red, Green, Blue, Amber, White] => 238,
            _ => {
                error!("Unmatched derby color state: {:?}.", self.0);
                0
            }
        }
    }
}

#[derive(Debug, Default)]
struct WhiteStrobe {
    state: GenericStrobe,
    /// 0 to 9
    program: usize,
}

impl WhiteStrobe {
    pub fn emit_state<F>(&self, emit: &mut F)
    where
        F: FnMut(WhiteStrobeStateChange),
    {
        use WhiteStrobeStateChange::*;
        emit(Program(self.program));
        let mut emit_general = |gsc| {
            emit(State(gsc));
        };
        self.state.emit_state(&mut emit_general);
    }

    pub fn handle_state_change(&mut self, sc: WhiteStrobeStateChange) -> Result<(), String> {
        use WhiteStrobeStateChange::*;
        match sc {
            State(g) => self.state.handle_state_change(g),
            Program(p) => {
                if p > 9 {
                    return Err(format!(
                        "swarmolon white strobe program index out of range: {}",
                        p
                    ));
                }
                self.program = p
            }
        }
        Ok(())
    }

    pub fn render(&self, group_controls: &FixtureGroupControls) -> u8 {
        let master_strobe = group_controls.strobe();
        if !self.state.on() || !master_strobe.state.on() {
            return 0;
        }
        let rate = if master_strobe.use_master_rate {
            master_strobe.state.rate()
        } else {
            self.state.rate()
        };
        let program_base = (self.program + 1) * 10;
        let program_speed = unipolar_to_range(9, 0, rate);
        program_base as u8 + program_speed
    }
}

#[derive(Clone, Copy, Debug)]
pub enum WhiteStrobeStateChange {
    /// Valid range is 0 to 9.
    Program(usize),
    State(GenericStrobeStateChange),
}
