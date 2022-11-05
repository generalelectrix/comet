//! Control abstractions that re-usable across fixture types.

use number::UnipolarFloat;

use crate::util::unipolar_to_range;

/// Most basic strobe control - active/not, plus rate.
#[derive(Default, Clone, Debug)]
pub struct GenericStrobe {
    on: bool,
    rate: UnipolarFloat,
}

impl GenericStrobe {
    pub fn on(&self) -> bool {
        self.on
    }

    pub fn rate(&self) -> UnipolarFloat {
        self.rate
    }

    pub fn emit_state<F>(&self, emit: &mut F)
    where
        F: FnMut(GenericStrobeStateChange),
    {
        use GenericStrobeStateChange::*;
        emit(On(self.on));
        emit(Rate(self.rate));
    }

    pub fn handle_state_change(&mut self, sc: GenericStrobeStateChange) {
        use GenericStrobeStateChange::*;
        match sc {
            On(v) => self.on = v,
            Rate(v) => self.rate = v,
        }
    }

    /// Render as a single DMX range with off.
    pub fn render_range(&self, off: u8, slow: u8, fast: u8) -> u8 {
        if self.on {
            unipolar_to_range(slow, fast, self.rate)
        } else {
            off
        }
    }

    /// Render as a single DMX range with off, using master as an override.
    /// Only strobe if master strobe is on and the local strobe is also on.
    /// Always use the master strobe rate.
    pub fn render_range_with_master(&self, master: &Self, off: u8, slow: u8, fast: u8) -> u8 {
        if self.on && master.on {
            unipolar_to_range(slow, fast, master.rate)
        } else {
            off
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GenericStrobeStateChange {
    On(bool),
    Rate(UnipolarFloat),
}
