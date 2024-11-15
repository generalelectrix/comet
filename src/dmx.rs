use std::{fmt::Display, ops::Add};

use serde::{Deserialize, Serialize};

/// A DMX address, indexed from 1.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone, Debug)]
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

impl Add<usize> for DmxAddr {
    type Output = DmxAddr;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

/// A data buffer for one DMX universe.
pub type DmxBuffer = [u8; 512];

/// Index into the DMX universes.
pub type UniverseIdx = usize;
