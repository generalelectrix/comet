//! Control for a generic strobe function.

use number::UnipolarFloat;

use crate::{
    fixture::control::{Bool, OscControl, Unipolar},
    util::unipolar_to_range,
};

#[derive(Debug)]
pub struct Strobe {
    on: Bool<()>,
    rate: Unipolar<()>,
}

impl Strobe {
    pub fn new(name: &str) -> Self {
        Self {
            on: Bool::new(format!("{name}On"), ()),
            rate: Unipolar::new(format!("{name}Rate"), ()),
        }
    }
}

impl OscControl<(bool, UnipolarFloat)> for Strobe {
    fn val(&self) -> (bool, UnipolarFloat) {
        (self.on.val(), self.rate.val())
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

impl Strobe {
    /// Render as a single DMX range, using master as an override.
    /// Only strobe if master strobe is on and the local strobe is also on.
    /// Return None if we're not strobing.
    pub fn render_range_with_master(
        &self,
        master: &crate::master::Strobe,
        slow: u8,
        fast: u8,
    ) -> Option<u8> {
        let rate = if master.use_master_rate {
            master.state.rate
        } else {
            self.rate.val()
        };
        if self.on.val() && master.state.on {
            Some(unipolar_to_range(slow, fast, rate))
        } else {
            None
        }
    }
}
