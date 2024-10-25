//! Basic control profile for 8-channel auto program control of the Chauvet
//! Freedom Stick.

//! Control profle for the Chauvet Rotosphere Q3, aka Son Of Spherion.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use super::color::Color;

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Debug)]
pub struct FreedomFries {
    dimmer: UnipolarChannel,
    color: Color,
    speed: UnipolarChannel,
    strobe: StrobeChannel,
    program: ProgramControl,
}

impl Default for FreedomFries {
    fn default() -> Self {
        Self {
            dimmer: Unipolar::full_channel("Dimmer", 0),
            color: Default::default(),
            speed: Unipolar::full_channel("Speed", 7),
            strobe: Strobe::channel("Strobe", 5, 0, 11, 255),

            program: ProgramControl::default(),
        }
    }
}

impl PatchAnimatedFixture for FreedomFries {
    const NAME: FixtureType = FixtureType("FreedomFries");
    fn channel_count(&self) -> usize {
        8
    }
}

impl AnimatedFixture for FreedomFries {
    type Target = AnimationTarget;
    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        self.dimmer
            .render(animation_vals.filter(&AnimationTarget::Dimmer), dmx_buf);
        self.speed
            .render(animation_vals.filter(&AnimationTarget::Speed), dmx_buf);
        self.color.render_without_animations(&mut dmx_buf[1..4]);
        dmx_buf[4] = 0;
        self.strobe
            .render_with_group(group_controls, std::iter::empty(), dmx_buf);
        self.program.render(std::iter::empty(), dmx_buf);
    }
}

impl ControllableFixture for FreedomFries {
    fn populate_controls(&mut self) {}

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.dimmer.emit_state(emitter);
        OscControl::emit_state(&self.color, emitter);
        self.speed.emit_state(emitter);
        self.strobe.emit_state(emitter);
        self.program.emit_state(emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        if self.dimmer.control(msg, emitter)? {
            return Ok(true);
        }
        if OscControl::control(&mut self.color, msg, emitter)? {
            return Ok(true);
        }
        if self.speed.control(msg, emitter)? {
            return Ok(true);
        }
        if self.strobe.control(msg, emitter)? {
            return Ok(true);
        }
        if self.program.control(msg, emitter)? {
            return Ok(true);
        }
        Ok(false)
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
    Speed,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        matches!(self, Self::Dimmer | Self::Speed)
    }
}

const PROGRAM_SELECT_LABEL: LabelArray = LabelArray {
    control: "ProgramLabel",
    n: 1,
    empty_label: "",
};

/// Control for indexed program select via a unipolar fader, with
/// value label read-out.
#[derive(Debug)]
struct ProgramControl {
    run_program: Bool<()>,
    select: Unipolar<()>,
    program_cycle_all: Bool<()>,
    selected: usize,
}

impl Default for ProgramControl {
    fn default() -> Self {
        Self {
            run_program: Bool::new_off("RunProgram", ()),
            select: Unipolar::new("Program", ()),
            program_cycle_all: Bool::new_off("ProgramCycleAll", ()),
            selected: 0,
        }
    }
}

impl ProgramControl {
    const PROGRAM_COUNT: usize = 27;
    const DMX_BUF_OFFSET: usize = 6;

    fn render(&self, _animations: impl Iterator<Item = f64>, dmx_buf: &mut [u8]) {
        dmx_buf[Self::DMX_BUF_OFFSET] = if !self.run_program.val() {
            0
        } else if self.program_cycle_all.val() {
            227
        } else {
            ((self.selected * 8) + 11) as u8
        };
    }
}

impl OscControl<()> for ProgramControl {
    fn control_direct(
        &mut self,
        _val: (),
        _emitter: &dyn crate::osc::EmitScopedOscMessage,
    ) -> anyhow::Result<()> {
        bail!("direct control is not implemented for ProgramControl");
    }

    fn emit_state(&self, emitter: &dyn crate::osc::EmitScopedOscMessage) {
        self.run_program.emit_state(emitter);
        self.select.emit_state(emitter);
        self.program_cycle_all.emit_state(emitter);
        PROGRAM_SELECT_LABEL.set([self.selected.to_string()].into_iter(), emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &dyn crate::osc::EmitScopedOscMessage,
    ) -> anyhow::Result<bool> {
        if self.run_program.control(msg, emitter)? {
            return Ok(true);
        }
        if self.program_cycle_all.control(msg, emitter)? {
            return Ok(true);
        }
        if self.select.control(msg, emitter)? {
            let new_val =
                unipolar_to_range(0, Self::PROGRAM_COUNT as u8 - 1, self.select.val()) as usize;
            if new_val >= Self::PROGRAM_COUNT {
                bail!(
                    "program select index {new_val} out of range (max {})",
                    Self::PROGRAM_COUNT
                );
            }
            self.selected =
                unipolar_to_range(0, Self::PROGRAM_COUNT as u8 - 1, self.select.val()) as usize;

            self.select.emit_state(emitter);
            PROGRAM_SELECT_LABEL.set([self.selected.to_string()].into_iter(), emitter);

            return Ok(true);
        }
        Ok(false)
    }
}
