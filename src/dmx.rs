use std::fmt::Display;

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

/// A data buffer for one DMX universe.
pub type DmxBuffer = [u8; 512];

/// Index into the DMX universes.
pub type UniverseIdx = usize;

/// The complete address of a fixture at a particular DMX address in a universe.
pub type FixtureAddress = (UniverseIdx, DmxAddr);
