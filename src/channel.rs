//! State and control definitions for fixture group channels.

use anyhow::{bail, Result};
use number::UnipolarFloat;
use serde::Deserialize;

use crate::fixture::{FixtureGroup, FixtureGroupKey, Patch};

/// The index of a channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct ChannelId(pub usize);

#[derive(Debug, Default)]
pub struct ChannelControls {
    pub level: UnipolarFloat,
}

impl ChannelControls {
    /// Return channel controls with level set to on.
    /// Used for fixtures that aren't wired up to actual channel selectors.
    pub const TRANSPARENT: Self = Self {
        level: UnipolarFloat::ONE,
    };
}

#[derive(Default)]
pub struct Channels {
    /// The control parameters for each channel.
    controls: Vec<ChannelControls>,
    /// Lookup from channel index to the fixture group assigned to that channel.
    channel_index: Vec<FixtureGroupKey>,
}

impl Channels {
    /// Add new channel controls, wired to the specified fixture.
    pub fn add(&mut self, group: FixtureGroupKey) -> ChannelId {
        let id = ChannelId(self.controls.len());
        self.controls.push(Default::default());
        self.channel_index.push(group);
        id
    }

    /// Validate that a channel index refers to a channel that actually exists.
    pub fn validate_channel(&self, channel: usize) -> Result<ChannelId> {
        if channel < self.channel_index.len() {
            Ok(ChannelId(channel))
        } else {
            bail!(
                "channel selector {channel} out of range, only {} channels are configured",
                self.channel_index.len()
            );
        }
    }

    /// Get the channel with the specified ID.
    pub fn get(&self, id: ChannelId) -> Option<&ChannelControls> {
        self.controls.get(id.0)
    }

    /// Iterate over all of the labels for each channels.
    pub fn channel_labels<'a>(&'a self, patch: &'a Patch) -> impl Iterator<Item = String> + 'a {
        self.channel_index
            .iter()
            .filter_map(|i| patch.get(i))
            .map(move |g| match g.name() {
                None => g.fixture_type().to_string(),
                Some(name) => format!("{name}({})", g.fixture_type()),
            })
    }

    /// Get a fixture group by channel ID.
    pub fn group_by_channel_mut<'a>(
        &self,
        patch: &'a mut Patch,
        channel: ChannelId,
    ) -> Result<&'a mut FixtureGroup> {
        let Some(fixture_key) = self.channel_index.get(channel.0) else {
            bail!("tried to get out-of-range channel {}.", channel.0);
        };
        if let Some(fixture) = patch.get_mut(fixture_key) {
            Ok(fixture)
        } else {
            bail!(
                "channel ID {} mapped to non-existent fixture key {fixture_key}",
                channel.0
            );
        }
    }
}
