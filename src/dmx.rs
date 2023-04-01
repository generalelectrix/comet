use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// A DMX address, indexed from 1.
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct DmxAddr(usize);

impl DmxAddr {
    /// Get the DMX buffer index of this address (indexed from 0).
    pub fn dmx_index(&self) -> usize {
        self.0 - 1
    }
}

impl Display for DmxAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
