//! Maintain UI state for animations.
use anyhow::{bail, Result};
use std::collections::HashMap;
use tunnels::animation::EmitStateChange as EmitAnimationStateChange;

use crate::{
    channel::Channels,
    fixture::{
        animation_target::{AnimationTargetIndex, ControllableTargetedAnimation, N_ANIM},
        Patch,
    },
    osc::{EmitControlMessage, HandleStateChange},
    show::ChannelId,
};

pub struct AnimationUIState {
    selected_animator_by_channel: HashMap<ChannelId, usize>,
}

impl AnimationUIState {
    pub fn new(initial_channel: Option<ChannelId>) -> Self {
        let mut state = Self {
            selected_animator_by_channel: Default::default(),
        };
        if let Some(channel) = initial_channel {
            state.selected_animator_by_channel.insert(channel, 0);
        }
        state
    }

    /// Emit all current animation state, including target and selection.
    pub fn emit_state(
        &self,
        channel: ChannelId,
        channels: &Channels,
        patch: &mut Patch,
        emitter: &dyn EmitControlMessage,
    ) -> anyhow::Result<()> {
        let (ta, index) = self.current_animation_with_index(channel, channels, patch)?;
        ta.anim().emit_state(&mut InnerAnimationEmitter(emitter));
        Self::emit(StateChange::Target(ta.target()), emitter);
        Self::emit(StateChange::SelectAnimation(index), emitter);
        Self::emit(StateChange::TargetLabels(ta.target_labels()), emitter);
        Ok(())
    }

    /// Handle a control message.
    pub fn control(
        &mut self,
        msg: ControlMessage,
        channel: ChannelId,
        channels: &Channels,
        patch: &mut Patch,
        emitter: &dyn EmitControlMessage,
    ) -> anyhow::Result<()> {
        match msg {
            ControlMessage::Animation(msg) => {
                self.current_animation(channel, channels, patch)?
                    .anim_mut()
                    .control(msg, &mut InnerAnimationEmitter(emitter));
            }
            ControlMessage::Target(msg) => {
                let anim = self.current_animation(channel, channels, patch)?;
                if anim.target() == msg {
                    return Ok(());
                }
                anim.set_target(msg)?;
                Self::emit(StateChange::Target(msg), emitter);
            }
            ControlMessage::SelectAnimation(n) => {
                if self.animation_index_for_channel(channel) == n {
                    return Ok(());
                }
                self.set_current_animation(channel, n)?;
                self.emit_state(channel, channels, patch, emitter)?;
            }
        }
        Ok(())
    }

    fn current_animation_with_index<'a>(
        &self,
        channel: ChannelId,
        channels: &'a Channels,
        patch: &'a mut Patch,
    ) -> Result<(&'a mut dyn ControllableTargetedAnimation, usize)> {
        let animation_index = self.animation_index_for_channel(channel);
        let group = channels.group_by_channel_mut(patch, channel)?;
        let key = group.key().clone();
        if let Some(anim) = group.get_animation(animation_index) {
            return Ok((anim, animation_index));
        }
        bail!("{key:?} does not have animations");
    }

    fn current_animation<'a>(
        &self,
        channel: ChannelId,
        channels: &'a Channels,
        patch: &'a mut Patch,
    ) -> Result<&'a mut dyn ControllableTargetedAnimation> {
        let (ta, _) = self.current_animation_with_index(channel, channels, patch)?;
        Ok(ta)
    }

    fn animation_index_for_channel(&self, channel: ChannelId) -> usize {
        self.selected_animator_by_channel
            .get(&channel)
            .cloned()
            .unwrap_or_default()
    }

    /// Set the current animation for the current channel to the provided value.
    pub fn set_current_animation(&mut self, channel: ChannelId, n: usize) -> anyhow::Result<()> {
        if n > N_ANIM {
            bail!("animator index {n} out of range");
        }
        self.selected_animator_by_channel.insert(channel, n);
        Ok(())
    }
}

struct InnerAnimationEmitter<'a>(&'a dyn EmitControlMessage);

impl<'a> EmitAnimationStateChange for InnerAnimationEmitter<'a> {
    fn emit_animation_state_change(&mut self, sc: tunnels::animation::StateChange) {
        AnimationUIState::emit(StateChange::Animation(sc), self.0);
    }
}

#[derive(Clone, Debug)]
pub enum ControlMessage {
    Animation(tunnels::animation::ControlMessage),
    Target(AnimationTargetIndex),
    SelectAnimation(usize),
}

#[derive(Clone, Debug)]
pub enum StateChange {
    Animation(tunnels::animation::StateChange),
    Target(AnimationTargetIndex),
    SelectAnimation(usize),
    TargetLabels(Vec<String>),
}
