//! Declarative fixture control models.
//! These types are intended to provide both a data model for fixture state,
//! as well as standardized ways to interact with that state.

use anyhow::Context;
use number::UnipolarFloat;

use crate::osc::{EmitScopedOscMessage, OscControlMessage, ScopedOscMessage};

mod unipolar;

pub use unipolar::*;

pub trait OscControl<T> {
    /// Return the OSC control name for this control.
    fn name(&self) -> &str;

    /// Handle an OSC message for setting this value.
    ///
    /// Return the new value if the message is handled successfully.
    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &dyn EmitScopedOscMessage,
    ) -> anyhow::Result<T>;

    /// Emit the current state of this control.
    /// Also return the current value for upstream processing.
    fn emit_state(&self, emitter: &dyn EmitScopedOscMessage) -> T;
}

pub trait RenderToDmxWithAnimations {
    /// Render a control into a DMX buffer.
    ///
    /// Handle animation values if any are provided.
    fn render(&self, animations: impl Iterator<Item = f64>, dmx_buf: &mut [u8]);
}

pub trait RenderToDmx<T> {
    /// Render a control into a DMX buffer using some strategy.
    fn render(&self, val: &T, dmx_buf: &mut [u8]);
}
