//! A control for unipolar floats.

use anyhow::Context;
use number::UnipolarFloat;

use crate::{
    osc::{EmitScopedOscMessage, OscControlMessage, ScopedOscMessage},
    util::unipolar_to_range,
};

use super::{OscControl, RenderToDmx, RenderToDmxWithAnimations};

/// A unipolar value, with controls.
#[derive(Debug)]
pub struct Unipolar<R: RenderToDmx<UnipolarFloat>> {
    val: UnipolarFloat,
    name: String,
    render: R,
}

impl<R: RenderToDmx<UnipolarFloat>> Unipolar<R> {
    /// Initialize a new control with the provided OSC control name.
    pub fn new<S: Into<String>>(osc_name: S, render: R) -> Self {
        Self {
            val: UnipolarFloat::ZERO,
            name: osc_name.into(),
            render,
        }
    }
}

impl<R: RenderToDmx<UnipolarFloat>> OscControl<UnipolarFloat> for Unipolar<R> {
    fn name(&self) -> &str {
        &self.name
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &dyn EmitScopedOscMessage,
    ) -> anyhow::Result<UnipolarFloat> {
        let v = msg.get_unipolar().with_context(|| self.name.clone())?;
        self.val = v;
        emitter.emit_float(&self.name, self.val.into());
        Ok(self.val)
    }

    fn emit_state(&self, emitter: &dyn EmitScopedOscMessage) -> UnipolarFloat {
        emitter.emit_float(&self.name, self.val.into());
        self.val
    }
}

impl<R: RenderToDmx<UnipolarFloat>> RenderToDmxWithAnimations for Unipolar<R> {
    fn render(&self, animations: impl Iterator<Item = f64>, dmx_buf: &mut [u8]) {
        let mut val = self.val.val();
        for anim_val in animations {
            // TODO: configurable blend modes
            val += anim_val;
        }
        // TODO: configurable coercing modes
        self.render.render(&UnipolarFloat::new(val), dmx_buf);
    }
}

/// Render a unipolar float to a continuous range.
#[derive(Debug)]
pub struct RenderUnipolarToRange {
    pub offset: usize,
    pub start: u8,
    pub end: u8,
}

impl RenderToDmx<UnipolarFloat> for RenderUnipolarToRange {
    fn render(&self, val: &UnipolarFloat, dmx_buf: &mut [u8]) {
        dmx_buf[self.offset] = unipolar_to_range(self.start, self.end, *val);
    }
}
