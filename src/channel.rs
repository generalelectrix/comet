//! State and control definitions for fixture group channels.

use number::UnipolarFloat;
use serde::Deserialize;

/// The index of a channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct ChannelId(pub usize);

pub struct Channel {
    level: UnipolarFloat,
}

pub struct Channels(Vec<Channel>);
