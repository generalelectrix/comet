//! TouchOSC array of unipolar fader.
use number::UnipolarFloat;
use rosc::{OscMessage, OscType};

use super::ControlMap;
use crate::fixture::ControlMessagePayload;
use anyhow::Result;

use anyhow::{anyhow, Context};

use crate::osc::get_unipolar;

/// Model a fader array.
#[derive(Clone)]
pub struct FaderArray {
    pub group: &'static str,
    pub control: &'static str,
}

impl FaderArray {
    /// Wire up this fader array to a control map.
    pub fn map<F>(self, map: &mut ControlMap<ControlMessagePayload>, process: F)
    where
        F: Fn(usize, UnipolarFloat) -> Result<ControlMessagePayload> + 'static + Copy,
    {
        map.add(self.group, self.control, move |msg| {
            let index = msg
                .addr_payload()
                .split('/')
                .skip(1)
                .take(1)
                .next()
                .ok_or_else(|| anyhow!("fader array index missing for {msg:?}"))?
                .parse::<usize>()
                .with_context(|| format!("handling message {msg:?}"))?;
            let val = get_unipolar(msg)?;
            process(index, val).map(Some)
        })
    }

    /// Emit state for a particular fader index.
    pub fn set<S>(&self, n: usize, val: UnipolarFloat, emitter: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        emitter.emit_osc(OscMessage {
            addr: format!("/{}/{}/{}", self.group, self.control, n),
            args: vec![OscType::Float(val.val() as f32)],
        });
    }
}
