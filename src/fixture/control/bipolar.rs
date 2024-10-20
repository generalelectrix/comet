//! A control for bipolar floats.

use anyhow::Context;
use number::BipolarFloat;

use crate::{
    osc::{EmitScopedOscMessage, OscControlMessage},
    util::{bipolar_to_range, bipolar_to_split_range},
};

use super::{OscControl, RenderToDmx, RenderToDmxWithAnimations};

/// A bipolar value, with controls.
#[derive(Debug)]
pub struct Bipolar<R: RenderToDmx<BipolarFloat>> {
    val: BipolarFloat,
    name: String,
    render: R,
}

/// A bipolar control that renders into a single DMX channel over a split range.
pub type BipolarSplitChannel = Bipolar<RenderBipolarToSplitRange>;

impl<R: RenderToDmx<BipolarFloat>> Bipolar<R> {
    /// Initialize a new control with the provided OSC control name.
    pub fn new<S: Into<String>>(name: S, render: R) -> Self {
        Self {
            val: BipolarFloat::ZERO,
            name: name.into(),
            render,
        }
    }
}

impl Bipolar<RenderBipolarToSplitRange> {
    /// Initialize a bipolar control that renders to a split range on a single DMX channel.
    ///
    /// This is the American DJ-style control.
    pub fn split_channel<S: Into<String>>(
        name: S,
        dmx_buf_offset: usize,
        cw_slow: u8,
        cw_fast: u8,
        ccw_slow: u8,
        ccw_fast: u8,
        stop: u8,
    ) -> Self {
        Self::new(
            name,
            RenderBipolarToSplitRange {
                dmx_buf_offset,
                cw_slow,
                cw_fast,
                ccw_slow,
                ccw_fast,
                stop,
            },
        )
    }
}

impl<R: RenderToDmx<BipolarFloat>> OscControl<BipolarFloat> for Bipolar<R> {
    fn name(&self) -> &str {
        &self.name
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &dyn EmitScopedOscMessage,
    ) -> anyhow::Result<Option<BipolarFloat>> {
        if msg.control() != self.name {
            return Ok(None);
        }
        let v = msg.get_bipolar().with_context(|| self.name.clone())?;
        self.val = v;
        emitter.emit_float(&self.name, self.val.into());
        Ok(Some(self.val))
    }

    fn emit_state(&self, emitter: &dyn EmitScopedOscMessage) -> BipolarFloat {
        emitter.emit_float(&self.name, self.val.into());
        self.val
    }
}

impl<R: RenderToDmx<BipolarFloat>> RenderToDmxWithAnimations for Bipolar<R> {
    fn render(&self, animations: impl Iterator<Item = f64>, dmx_buf: &mut [u8]) {
        let mut val = self.val.val();
        for anim_val in animations {
            // TODO: configurable blend modes
            val += anim_val;
        }
        // TODO: configurable coercing modes
        self.render.render(&BipolarFloat::new(val), dmx_buf);
    }
}

/// Render a bipolar float to a continuous range.
#[derive(Debug)]
pub struct RenderBipolarToSplitRange {
    pub dmx_buf_offset: usize,
    pub cw_slow: u8,
    pub cw_fast: u8,
    pub ccw_slow: u8,
    pub ccw_fast: u8,
    pub stop: u8,
}

impl RenderToDmx<BipolarFloat> for RenderBipolarToSplitRange {
    fn render(&self, val: &BipolarFloat, dmx_buf: &mut [u8]) {
        dmx_buf[self.dmx_buf_offset] = bipolar_to_split_range(
            *val,
            self.cw_slow,
            self.cw_fast,
            self.ccw_slow,
            self.ccw_fast,
            self.stop,
        );
    }
}
