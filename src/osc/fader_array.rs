//! TouchOSC array of unipolar fader.
use number::UnipolarFloat;
use rosc::{OscMessage, OscType};

use super::{GroupControlMap, ScopedOscMessage};
use anyhow::{bail, Result};

use anyhow::{anyhow, Context};

use crate::osc::get_unipolar;

/// Model a fader array.
#[derive(Clone)]
pub struct FaderArray {
    pub control: &'static str,
}

impl FaderArray {
    /// Wire up this fader array to a control map.
    pub fn map<F, T>(self, map: &mut GroupControlMap<T>, process: F)
    where
        F: Fn(usize, UnipolarFloat) -> Result<T> + 'static + Copy,
    {
        map.add(self.control, move |msg| {
            let index = msg
                .addr_payload()
                .split('/')
                .skip(1)
                .take(1)
                .next()
                .ok_or_else(|| anyhow!("fader array index missing for {msg:?}"))?
                .parse::<usize>()
                .with_context(|| format!("handling message {msg:?}"))?;
            if index == 0 {
                bail!("fader array index is 0: {msg:?}");
            }
            let val = get_unipolar(msg)?;
            process(index - 1, val).map(Some)
        })
    }

    /// Emit state for a particular fader index.
    pub fn set<S>(&self, n: usize, val: UnipolarFloat, emitter: &S)
    where
        S: crate::osc::EmitScopedOscMessage + ?Sized,
    {
        emitter.emit_osc(ScopedOscMessage {
            control: &format!("/{}/{}", self.control, n + 1),
            arg: OscType::Float(val.val() as f32),
        });
    }
}
