//! Declarative fixture control models.
//! These types are intended to provide both a data model for fixture state,
//! as well as standardized ways to interact with that state.

use crate::osc::{EmitScopedOscMessage, OscControlMessage};

mod bipolar;
mod bool;
mod unipolar;

pub use bipolar::*;
pub use bool::*;
pub use unipolar::*;

pub trait OscControl<T> {
    /// Return the current state of this control.
    fn val(&self) -> T;

    /// Potentially handle an OSC control message.
    /// If we handle the message, return true.
    /// If we don't handle the message, return false.
    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &dyn EmitScopedOscMessage,
    ) -> anyhow::Result<bool>;

    /// Emit the current state of this control.
    fn emit_state(&self, emitter: &dyn EmitScopedOscMessage);
}

pub trait RenderToDmxWithAnimations {
    /// Render a control into a DMX buffer.
    ///
    /// Handle animation values if any are provided.
    fn render(&self, animations: impl Iterator<Item = f64>, dmx_buf: &mut [u8]);

    /// Render a control into a DMX buffer, without any animations.
    fn render_no_anim(&self, dmx_buf: &mut [u8]) {
        self.render(std::iter::empty(), dmx_buf);
    }
}

pub trait RenderToDmx<T> {
    /// Render a control into a DMX buffer using some strategy.
    fn render(&self, val: &T, dmx_buf: &mut [u8]);
}

/// A render strategy that does nothing.
/// Used for controls which themselves are not rendered directly to DMX.
impl<T> RenderToDmx<T> for () {
    fn render(&self, _val: &T, _dmx_buf: &mut [u8]) {}
}
