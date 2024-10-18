//! State and control definitions for fixture group channels.

use std::{collections::HashMap, fmt::Display};

use anyhow::{anyhow, bail, Context, Result};
use log::error;
use number::UnipolarFloat;
use serde::Deserialize;

use crate::{
    fixture::{FixtureGroup, FixtureGroupKey, Patch},
    osc::{EmitControlMessage, EmitOscMessage, HandleStateChange},
};

/// The index of a channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct ChannelId(usize);

impl From<ChannelId> for usize {
    fn from(value: ChannelId) -> Self {
        value.0
    }
}

impl Display for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Default)]
pub struct Channels {
    /// Lookup from channel index to the fixture group assigned to that channel.
    channel_index: Vec<FixtureGroupKey>,
    /// Reverse-lookup from fixture group key to channel index.
    fixture_channel_index: HashMap<FixtureGroupKey, ChannelId>,
    /// The channel ID that is currently selected.
    current_channel: Option<ChannelId>,
}

impl Channels {
    /// Add new channel controls, wired to the specified fixture.
    pub fn add(&mut self, group: FixtureGroupKey) -> ChannelId {
        let id = ChannelId(self.channel_index.len());
        self.channel_index.push(group.clone());
        self.fixture_channel_index.insert(group, id);
        id
    }

    /// Iterate over valid channel IDs.
    pub fn channel_ids(&self) -> impl Iterator<Item = ChannelId> + '_ {
        self.channel_index
            .iter()
            .enumerate()
            .map(|(i, _)| ChannelId(i))
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

    /// Look up a channel ID by fixture group key.
    pub fn channel_for_fixture(&self, group: &FixtureGroupKey) -> Option<ChannelId> {
        self.fixture_channel_index.get(group).cloned()
    }

    /// Iterate over all of the labels for each channels.
    pub fn channel_labels<'a>(&'a self, patch: &'a Patch) -> impl Iterator<Item = String> + 'a {
        self.channel_index
            .iter()
            .filter_map(|i| match patch.get(i) {
                Ok(f) => Some(f),
                Err(err) => {
                    error!("Patch inconsistency generating channel labels: {err}");
                    None
                }
            })
            .map(move |g| match g.name() {
                None => g.fixture_type().to_string(),
                Some(name) => format!("{name}({})", g.fixture_type()),
            })
    }

    /// Get a fixture group by channel ID.
    pub fn group_by_channel<'a>(
        &self,
        patch: &'a Patch,
        channel: ChannelId,
    ) -> Result<&'a FixtureGroup> {
        let Some(fixture_key) = self.channel_index.get(channel.0) else {
            bail!("tried to get out-of-range channel {channel}");
        };
        patch
            .get(fixture_key)
            .with_context(|| format!("channel {channel}"))
    }

    /// Get a fixture group by channel ID, mutably.
    pub fn group_by_channel_mut<'a>(
        &self,
        patch: &'a mut Patch,
        channel: ChannelId,
    ) -> Result<&'a mut FixtureGroup> {
        let Some(fixture_key) = self.channel_index.get(channel.0) else {
            bail!("tried to get out-of-range channel {channel}");
        };
        patch
            .get_mut(fixture_key)
            .with_context(|| format!("channel {channel}"))
    }

    pub fn current_channel(&self) -> Option<ChannelId> {
        self.current_channel
    }

    /// Emit all current channel state.
    pub fn emit_state(
        &self,
        selected_fixture_only: bool,
        patch: &mut Patch,
        emitter: &dyn EmitControlMessage,
    ) {
        if let Some(channel) = self.current_channel {
            Self::emit(StateChange::SelectChannel(channel), emitter);
        }
        Self::emit(
            StateChange::ChannelLabels(self.channel_labels(patch).collect()),
            emitter,
        );
        if selected_fixture_only {
            if let Some(channel_id) = self.current_channel {
                match self.group_by_channel(patch, channel_id) {
                    Ok(f) => f.emit_state(ChannelStateEmitter {
                        channel_id: Some(channel_id),
                        emitter,
                    }),
                    Err(err) => error!("Failed to emit channel {channel_id} state: {err}."),
                }
            }
        } else {
            for channel_id in self.channel_ids() {
                match self.group_by_channel(patch, channel_id) {
                    Ok(f) => f.emit_state(ChannelStateEmitter {
                        channel_id: Some(channel_id),
                        emitter,
                    }),
                    Err(err) => error!("Failed to emit channel {channel_id} state: {err}."),
                }
            }
        }
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
                self.emit_state(true, patch, emitter);
            }
            ControlMessage::Control { channel_id, msg } => {
                let channel_id = if let Some(id) = channel_id {
                    self.validate_channel(id)?
                } else {
                    self.current_channel.ok_or_else(||
                        anyhow!("no channel ID provided or selected for channel control message {msg:?}")
                    )?
                };
                self.group_by_channel_mut(patch, channel_id)?
                    .control_from_channel(
                        &msg,
                        ChannelStateEmitter {
                            channel_id: Some(channel_id),
                            emitter,
                        },
                    );
            }
        }
        Ok(())
    }
}

/// Provide methods to emit channel control state changes for a specific channel.
/// If no channel is set, no state change events will be emitted.
#[derive(Clone, Copy)]
pub struct ChannelStateEmitter<'a> {
    channel_id: Option<ChannelId>,
    emitter: &'a dyn EmitControlMessage,
}

impl<'a> ChannelStateEmitter<'a> {
    /// An emitter that ignores channel state changes.
    pub fn new(channel_id: Option<ChannelId>, emitter: &'a dyn EmitControlMessage) -> Self {
        Self {
            channel_id,
            emitter,
        }
    }

    /// Emit the provided state change.
    pub fn emit(&self, msg: ChannelStateChange) {
        let Some(channel_id) = self.channel_id else {
            return;
        };
        Channels::emit(StateChange::State { channel_id, msg }, self.emitter);
    }
}

impl<'a> EmitOscMessage for ChannelStateEmitter<'a> {
    fn emit_osc(&self, msg: rosc::OscMessage) {
        self.emitter.emit_osc(msg);
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
