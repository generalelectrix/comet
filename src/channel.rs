//! State and control definitions for fixture group channels.

use anyhow::{anyhow, bail, Result};
use number::UnipolarFloat;
use serde::Deserialize;

use crate::{
    fixture::{FixtureGroup, FixtureGroupKey, Patch},
    osc::EmitControlMessage,
    osc::HandleStateChange,
};

/// The index of a channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct ChannelId(pub usize);

#[derive(Default)]
pub struct Channels {
    /// Lookup from channel index to the fixture group assigned to that channel.
    channel_index: Vec<FixtureGroupKey>,
    /// The channel ID that is currently selected.
    current_channel: Option<ChannelId>,
}

impl Channels {
    /// Add new channel controls, wired to the specified fixture.
    pub fn add(&mut self, group: FixtureGroupKey) -> ChannelId {
        let id = ChannelId(self.channel_index.len());
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

    pub fn current_channel(&self) -> Option<ChannelId> {
        self.current_channel
    }

    /// Emit all current channel state.
    pub fn emit_state(&self, patch: &mut Patch, emitter: &dyn EmitControlMessage) {
        if let Some(channel) = self.current_channel {
            Self::emit(StateChange::SelectChannel(channel), emitter);
        }
        Self::emit(
            StateChange::ChannelLabels(self.channel_labels(patch).collect()),
            emitter,
        );
        todo!("emit fixture channel control state");
    }

    /// Handle a control message.
    pub fn control(
        &mut self,
        msg: ControlMessage,
        patch: &mut Patch,
        emitter: &dyn EmitControlMessage,
    ) -> anyhow::Result<()> {
        match msg {
            ControlMessage::SelectChannel(g) => {
                // Validate the channel.
                let channel = self.validate_channel(g)?;
                if self.current_channel == Some(channel) {
                    // Channel is not changed, ignore.
                    return Ok(());
                }
                self.current_channel = Some(channel);
                self.emit_state(patch, emitter);
                Ok(())
            }
            ControlMessage::Control { channel_id, msg } => {
                let channel_id = if let Some(id) = channel_id {
                    self.validate_channel(id)?
                } else {
                    self.current_channel.ok_or_else(||
                        anyhow!("no channel ID provided or selected for channel control message {msg:?}")
                    )?
                };
                let target_fixture = &self.channel_index[channel_id.0];
                let Some(target_fixture) = patch.get_mut(target_fixture) else {
                    bail!("fixture key {target_fixture:?} assigned to channel {} unexpectedly missing from patch", channel_id.0);
                };
                let channel_emitter = ChannelStateEmitter {
                    channel_id,
                    emitter,
                };
                target_fixture.control_from_channel(&msg, &channel_emitter)
            }
        }
    }
}

/// Provide methods to emit channel control state changes for a specific channel.
pub struct ChannelStateEmitter<'a> {
    channel_id: ChannelId,
    emitter: &'a dyn EmitControlMessage,
}

impl<'a> ChannelStateEmitter<'a> {
    /// Return the underlying control message emitter.
    pub fn raw_emitter(&self) -> &dyn EmitControlMessage {
        self.emitter
    }
}

#[derive(Clone, Debug)]
pub enum ControlMessage {
    SelectChannel(usize),
    Control {
        channel_id: Option<usize>,
        msg: ChannelControlMessage,
    },
}

#[derive(Clone, Debug)]
pub enum StateChange {
    SelectChannel(ChannelId),
    ChannelLabels(Vec<String>),
    State {
        channel_id: ChannelId,
        msg: ChannelStateChange,
    },
}

#[derive(Clone, Debug)]
pub enum ChannelStateChange {
    Level(UnipolarFloat),
}

pub type ChannelControlMessage = ChannelStateChange;
