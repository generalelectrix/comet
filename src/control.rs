//! Top-level traits and types for control events.

use crate::osc::{EmitOscMessage, EmitScopedOscMessage, HandleOscStateChange};

/// Emit scoped control messages.
/// Will be extended in the future to potentially cover more cases.
pub trait EmitScopedControlMessage: EmitScopedOscMessage {}

impl<T> EmitScopedControlMessage for T where T: EmitScopedOscMessage {}

/// Emit control messages.
/// Will be extended in the future to potentially cover more cases.
pub trait EmitControlMessage: EmitOscMessage {}

impl<T> EmitControlMessage for T where T: EmitOscMessage {}

/// Process a state change message into control state changes.
pub trait HandleStateChange<SC>: HandleOscStateChange<SC> {
    fn emit<S>(sc: SC, send: &S)
    where
        S: EmitScopedControlMessage + ?Sized,
    {
        Self::emit_osc_state_change(sc, send);
    }
}

impl<T, SC> HandleStateChange<SC> for T where T: HandleOscStateChange<SC> {}

pub mod prelude {
    pub use super::*;
    pub use crate::osc::prelude::*;
}
