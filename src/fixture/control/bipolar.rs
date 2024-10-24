//! A control for bipolar floats.

use anyhow::Context;
use number::BipolarFloat;

use crate::{
    osc::{EmitScopedOscMessage, OscControlMessage},
    util::{bipolar_to_range, bipolar_to_split_range},
};

use super::{Bool, OscControl, RenderToDmx, RenderToDmxWithAnimations};

/// A bipolar value, with controls.
#[derive(Debug)]
pub struct Bipolar<R: RenderToDmx<BipolarFloat>> {
    val: BipolarFloat,
    name: String,
    render: R,
}

/// A bipolar control that renders into a single DMX channel over a split range.
pub type BipolarSplitChannel = Bipolar<RenderBipolarToSplitRange>;

/// Bipolar split channel with mirroring.
pub type BipolarSplitChannelMirror = Mirrored<RenderBipolarToSplitRange>;

/// A bipolar control that renders into a single DMX channel over a continuous range.
pub type BipolarChannel = Bipolar<RenderBipolarToRange>;

/// Bipolar continuous channel with mirroring.
pub type BipolarChannelMirror = Mirrored<RenderBipolarToRange>;

impl<R: RenderToDmx<BipolarFloat>> Bipolar<R> {
    /// Initialize a new control with the provided OSC control name.
    pub fn new<S: Into<String>>(name: S, render: R) -> Self {
        Self {
            val: BipolarFloat::ZERO,
            name: name.into(),
            render,
        }
    }

    /// Decorate this control with automatic mirroring.
    pub fn with_mirroring(self) -> Mirrored<R> {
        Mirrored::new(self)
    }

    fn val_with_anim(&self, animations: impl Iterator<Item = f64>) -> BipolarFloat {
        let mut val = self.val.val();
        for anim_val in animations {
            // TODO: configurable blend modes
            val += anim_val;
        }
        BipolarFloat::new(val)
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

impl Bipolar<RenderBipolarToRange> {
    /// Initialize a bipolar control that renders to a continuous range on a single DMX channel.
    pub fn channel<S: Into<String>>(name: S, dmx_buf_offset: usize, start: u8, end: u8) -> Self {
        Self::new(
            name,
            RenderBipolarToRange {
                dmx_buf_offset,
                start,
                end,
            },
        )
    }
}

impl<R: RenderToDmx<BipolarFloat>> OscControl<BipolarFloat> for Bipolar<R> {
    fn val(&self) -> &BipolarFloat {
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
        let v = msg.get_bipolar().with_context(|| self.name.clone())?;
        self.val = v;
        emitter.emit_float(&self.name, self.val.into());
        Ok(true)
    }

    fn emit_state(&self, emitter: &dyn EmitScopedOscMessage) {
        emitter.emit_float(&self.name, self.val.into());
    }
}

impl<R: RenderToDmx<BipolarFloat>> RenderToDmxWithAnimations for Bipolar<R> {
    fn render(&self, animations: impl Iterator<Item = f64>, dmx_buf: &mut [u8]) {
        // TODO: configurable coercing modes
        self.render.render(&self.val_with_anim(animations), dmx_buf);
    }
}

/// Render a bipolar float to a split range.
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

/// Render a bipolar float to a continuous range.
#[derive(Debug)]
pub struct RenderBipolarToRange {
    pub dmx_buf_offset: usize,
    pub start: u8,
    pub end: u8,
}

impl RenderToDmx<BipolarFloat> for RenderBipolarToRange {
    fn render(&self, val: &BipolarFloat, dmx_buf: &mut [u8]) {
        dmx_buf[self.dmx_buf_offset] = bipolar_to_range(self.start, self.end, *val);
    }
}

/// A decorator for bipolar controls that adds configurable mirroring.
///
/// "Mirror" is pre-pended to the inner control name to create the mirror
/// activation toggle. Since auto-mirroring is awesome, this defaults to on,
/// so that control layouts will by default respect the patch-level mirroring
/// settings even if no explicit mirror buttons are included in the layout.
#[derive(Debug)]
pub struct Mirrored<R: RenderToDmx<BipolarFloat>> {
    control: Bipolar<R>,
    mirror: Bool<()>,
}

impl<R: RenderToDmx<BipolarFloat>> Mirrored<R> {
    /// Decorate a bipolar float control with auto-mirroring.
    fn new(control: Bipolar<R>) -> Self {
        Self {
            mirror: Bool::new_on(format!("Mirror{}", control.name), ()),
            control,
        }
    }
}

impl<R: RenderToDmx<BipolarFloat>> RenderToDmxWithAnimations for Mirrored<R> {
    fn render(&self, animations: impl Iterator<Item = f64>, dmx_buf: &mut [u8]) {
        // FIXME: should strive to eliminate this code path
        // ignores mirroring when no group controls provided
        self.control.render(animations, dmx_buf);
    }

    fn render_with_group(
        &self,
        group_controls: &crate::fixture::FixtureGroupControls,
        animations: impl Iterator<Item = f64>,
        dmx_buf: &mut [u8],
    ) {
        self.control.render.render(
            &self
                .control
                .val_with_anim(animations)
                .invert_if(group_controls.mirror && *self.mirror.val()),
            dmx_buf,
        );
    }
}

// Since we can't know the mirroring state at this level, we don't expose the
// current value of this control.
impl<R: RenderToDmx<BipolarFloat>> OscControl<()> for Mirrored<R> {
    fn val(&self) -> &() {
        &()
    }

    fn emit_state(&self, emitter: &dyn EmitScopedOscMessage) {
        self.control.emit_state(emitter);
        self.mirror.emit_state(emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &dyn EmitScopedOscMessage,
    ) -> anyhow::Result<bool> {
        if self.control.control(msg, emitter)? {
            return Ok(true);
        }
        if self.mirror.control(msg, emitter)? {
            return Ok(true);
        }
        Ok(false)
    }
}
