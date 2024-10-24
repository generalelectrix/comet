//! A control for phases.

use anyhow::Context;
use number::{Phase, UnipolarFloat};

use crate::{
    osc::{EmitScopedOscMessage, OscControlMessage},
    util::unipolar_to_range,
};

use super::{OscControl, RenderToDmx, RenderToDmxWithAnimations};

/// A phase value, with controls.
#[derive(Debug)]
pub struct PhaseControl<R: RenderToDmx<Phase>> {
    val: Phase,
    name: String,
    render: R,
}

/// A phase control that renders into a single DMX channel over a range.
pub type PhaseChannel = PhaseControl<RenderPhaseToRange>;

impl<R: RenderToDmx<Phase>> PhaseControl<R> {
    /// Initialize a new control with the provided OSC control name.
    pub fn new<S: Into<String>>(name: S, render: R) -> Self {
        Self {
            val: Phase::ZERO,
            name: name.into(),
            render,
        }
    }
}

impl PhaseControl<RenderPhaseToRange> {
    /// Initialize a phase control that renders to a full DMX channel.
    pub fn full_channel<S: Into<String>>(name: S, dmx_buf_offset: usize) -> Self {
        Self::new(
            name,
            RenderPhaseToRange {
                dmx_buf_offset,
                start: 0,
                end: 255,
            },
        )
    }
}

impl<R: RenderToDmx<Phase>> OscControl<Phase> for PhaseControl<R> {
    fn val(&self) -> &Phase {
        &self.val
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &dyn EmitScopedOscMessage,
    ) -> anyhow::Result<bool> {
        if msg.control() != self.name {
            return Ok(false);
        }
        let v = msg.get_phase().with_context(|| self.name.clone())?;
        self.val = v;
        emitter.emit_float(&self.name, self.val.into());
        Ok(true)
    }

    fn emit_state(&self, emitter: &dyn EmitScopedOscMessage) {
        emitter.emit_float(&self.name, self.val.into());
    }
}

impl<R: RenderToDmx<Phase>> RenderToDmxWithAnimations for PhaseControl<R> {
    fn render(&self, animations: impl Iterator<Item = f64>, dmx_buf: &mut [u8]) {
        let mut val = self.val.val();
        for anim_val in animations {
            // TODO: configurable blend modes
            val += anim_val;
        }
        self.render.render(&Phase::new(val), dmx_buf);
    }
}

/// Render a phase float to a continuous range.
#[derive(Debug)]
pub struct RenderPhaseToRange {
    pub dmx_buf_offset: usize,
    pub start: u8,
    pub end: u8,
}

impl RenderToDmx<Phase> for RenderPhaseToRange {
    fn render(&self, val: &Phase, dmx_buf: &mut [u8]) {
        dmx_buf[self.dmx_buf_offset] =
            unipolar_to_range(self.start, self.end, UnipolarFloat::new(val.val()));
    }
}
