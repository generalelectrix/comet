//! Control profle for the Chauvet Swarm 5 FX, aka the Swarmolon.
//! Also

use log::error;
use number::{BipolarFloat, UnipolarFloat};

use super::generic::{GenericStrobe, GenericStrobeStateChange};
use super::{EmitFixtureStateChange, Fixture, FixtureControlMessage, PatchFixture};
use crate::master::MasterControls;
use crate::osc::{quadratic_bipolar, quartic_bipolar};
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
}

impl PatchFixture for Swarmolon {
    fn channel_count(&self) -> usize {
        if self.quad_phase_mindmeld {
            13
        } else {
            9
        }
    }

    fn new(
        options: &std::collections::HashMap<String, String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut s = Self::default();
        if options.contains_key("quad_phase") {
            s.quad_phase_mindmeld = true;
        }
        Ok(s)
    }
}

impl Swarmolon {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitFixtureStateChange) {
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
        emitter.emit_swarmolon(sc);
    }
}

impl Fixture for Swarmolon {
    fn render(&self, master: &MasterControls, dmx_buf: &mut [u8]) {
        dmx_buf[0] = 255; // always set to DMX mode
        dmx_buf[1] = self.derby_color.render();
        dmx_buf[2] = 0; // Not using automatic derby programs.
        dmx_buf[3] = self
            .derby_strobe
            .render_range_with_master(master.strobe(), 0, 254, 10);
        dmx_buf[4] = self.white_strobe.render(master);
        dmx_buf[5] = match (self.red_laser_on, self.green_laser_on) {
            (false, false) => 0,
            (true, false) => 10,
            (false, true) => 50,
            (true, true) => 255, // TODO: verify this is actually correct.
        };
        dmx_buf[6] = self
            .laser_strobe
            .render_range_with_master(master.strobe(), 0, 5, 254);
        dmx_buf[7] = bipolar_to_split_range(self.derby_rotation, 5, 127, 134, 255, 0);
        dmx_buf[8] = bipolar_to_split_range(self.laser_rotation, 5, 127, 134, 255, 0);
        if self.quad_phase_mindmeld {
            let color_val = self.derby_color.render_quad_phase();
            dmx_buf[9] = color_val;
            dmx_buf[10] =
                bipolar_to_split_range(quartic_bipolar(self.derby_rotation), 120, 10, 135, 245, 0);
            dmx_buf[11] = self
                .derby_strobe
                .render_range_with_master(master.strobe(), 0, 1, 255);
            dmx_buf[12] = if color_val == 0 { 0 } else { 255 };
        }
    }

    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        use StateChange::*;
        self.derby_color.emit_state(emitter);
        let mut emit_derby_strobe = |ssc| {
            emitter.emit_swarmolon(DerbyStrobe(ssc));
        };
        self.derby_strobe.emit_state(&mut emit_derby_strobe);
        emitter.emit_swarmolon(DerbyRotation(self.derby_rotation));
        let mut emit_white_strobe = |ssc| {
            emitter.emit_swarmolon(WhiteStrobe(ssc));
        };
        self.white_strobe.emit_state(&mut emit_white_strobe);
        emitter.emit_swarmolon(RedLaserOn(self.red_laser_on));
        emitter.emit_swarmolon(GreenLaserOn(self.green_laser_on));
        let mut emit_laser_strobe = |ssc| {
            emitter.emit_swarmolon(LaserStrobe(ssc));
        };
        self.laser_strobe.emit_state(&mut emit_laser_strobe);
        emitter.emit_swarmolon(LaserRotation(self.laser_rotation));
    }

    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::Swarmolon(msg) => {
                match msg {
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

                None
            }
            other => Some(other),
        }
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

    pub fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange) {
        for color in DerbyColor::iter() {
            let state = self.0.contains(&color);
            emitter.emit_swarmolon(StateChange::DerbyColor(color, state));
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

    pub fn render(&self, master: &MasterControls) -> u8 {
        if !self.state.on() || !master.strobe().on() {
            return 0;
        }
        let program_base = (self.program + 1) * 10;
        let program_speed = unipolar_to_range(9, 0, master.strobe().rate());
        program_base as u8 + program_speed
    }
}

#[derive(Clone, Copy, Debug)]
pub enum WhiteStrobeStateChange {
    /// Valid range is 0 to 9.
    Program(usize),
    State(GenericStrobeStateChange),
}
