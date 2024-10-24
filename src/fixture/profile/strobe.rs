//! Control for a generic strobe function.

use number::UnipolarFloat;

use crate::{
    fixture::control::{Bool, OscControl, RenderToDmx, RenderToDmxWithAnimations, Unipolar},
    util::unipolar_to_range,
};

/// Generic strobe control, using unipolar rate.
/// Usually also listens to the master strobe control parameter.
#[derive(Debug)]
pub struct Strobe<R: RenderToDmx<Option<UnipolarFloat>>> {
    on: Bool<()>,
    rate: Unipolar<()>,
    render: R,
}

/// A strobe controlling a single basic DMX channel.
pub type StrobeChannel = Strobe<RenderStrobeToRange>;

impl<R: RenderToDmx<Option<UnipolarFloat>>> Strobe<R> {
    pub fn new(name: &str, render: R) -> Self {
        Self {
            on: Bool::new(format!("{name}On"), ()),
            rate: Unipolar::new(format!("{name}Rate"), ()),
            render,
        }
    }

    /// Get the current value of this strobe control, if active.
    pub fn val_with_master(&self, master: &crate::master::Strobe) -> Option<UnipolarFloat> {
        let rate = if master.use_master_rate {
            master.state.rate
        } else {
            *self.rate.val()
        };

        (*self.on.val() && master.state.on).then_some(rate)
    }
}

impl StrobeChannel {
    /// Create a strobe that renders to DMX as a single channel, with provided bounds.
    pub fn channel(name: &str, dmx_buf_offset: usize, slow: u8, fast: u8, stop: u8) -> Self {
        Self::new(
            name,
            RenderStrobeToRange {
                dmx_buf_offset,
                slow,
                fast,
                stop,
            },
        )
    }
}

impl<R: RenderToDmx<Option<UnipolarFloat>>> OscControl<()> for Strobe<R> {
    fn val(&self) -> &() {
        &()
    }

    fn control(
        &mut self,
        msg: &crate::osc::OscControlMessage,
        emitter: &dyn crate::osc::EmitScopedOscMessage,
    ) -> anyhow::Result<bool> {
        if self.on.control(msg, emitter)? {
            return Ok(true);
        }
        if self.rate.control(msg, emitter)? {
            return Ok(true);
        }
        Ok(false)
    }

    fn emit_state(&self, emitter: &dyn crate::osc::EmitScopedOscMessage) {
        self.on.emit_state(emitter);
        self.rate.emit_state(emitter);
    }
}

impl<R: RenderToDmx<Option<UnipolarFloat>>> RenderToDmxWithAnimations for Strobe<R> {
    fn render(&self, _animations: impl Iterator<Item = f64>, dmx_buf: &mut [u8]) {
        // FIXME: need to tweak traits around to avoid the need for this
        if *self.on.val() {
            self.render.render(&Some(*self.rate.val()), dmx_buf);
        } else {
            self.render.render(&None, dmx_buf);
        }
    }

    fn render_with_group(
        &self,
        group_controls: &crate::fixture::FixtureGroupControls,
        _animations: impl Iterator<Item = f64>,
        dmx_buf: &mut [u8],
    ) {
        self.render
            .render(&self.val_with_master(group_controls.strobe()), dmx_buf);
    }
}

#[derive(Debug)]
pub struct RenderStrobeToRange {
    dmx_buf_offset: usize,
    slow: u8,
    fast: u8,
    stop: u8,
}

impl RenderToDmx<Option<UnipolarFloat>> for RenderStrobeToRange {
    fn render(&self, val: &Option<UnipolarFloat>, dmx_buf: &mut [u8]) {
        dmx_buf[self.dmx_buf_offset] = if let Some(rate) = *val {
            unipolar_to_range(self.slow, self.fast, rate)
        } else {
            self.stop
        }
    }
}
