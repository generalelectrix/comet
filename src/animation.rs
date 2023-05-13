//! Maintain UI state for animations.
use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;
use tunnels::animation::EmitStateChange as EmitAnimationStateChange;

use crate::{
    fixture::{
        animation_target::{AnimationTargetIndex, ControllableTargetedAnimation},
        EmitStateChange, FixtureGroupKey, FixtureStateChange, GroupName, Patch,
        StateChange as FixtureStateChangeWithGroup, N_ANIM,
    },
    osc::OscController,
};

#[derive(Default)]
pub struct AnimationUIState {
    pub current_group: Option<FixtureGroupKey>,
    pub selected_animator_by_group: HashMap<FixtureGroupKey, usize>,
}

impl AnimationUIState {
    /// Emit all current animation state, including target and selection.
    pub fn emit_state(
        &self,
        patch: &mut Patch,                  // FIXME this doesn't need to be mutable
        osc_controller: &mut OscController, // FIXME this ought to be generic
    ) -> Result<()> {
        let (ta, index) = self.current_animation_with_index(patch)?;
        ta.animation.emit_state(osc_controller);
        osc_controller.emit(FixtureStateChangeWithGroup {
            group: GroupName::none(),
            sc: FixtureStateChange::AnimationTarget(ta.target),
        });
        osc_controller.emit(FixtureStateChangeWithGroup {
            group: GroupName::none(),
            sc: FixtureStateChange::AnimationSelect(index),
        });
        Ok(())
    }

    /// Handle a control message.
    pub fn control(
        &mut self,
        msg: ControlMessage,
        patch: &mut Patch,
        osc_controller: &mut OscController,
    ) {
        match msg {
            ControlMessage::Animation(msg) => {
                self.current_animation(patch)?
                    .animation
                    .control(msg, osc_controller);
            }
            ControlMessage::Target(msg) => {
                self.current_animation(patch)?.target = msg;
                osc_controller.emit(FixtureStateChangeWithGroup {
                    group: GroupName::none(),
                    sc: crate::fixture::FixtureStateChange::AnimationTarget(msg),
                });
            }
            ControlMessage::Select(n) => {
                if self.animation_index_for_key(self.current_group()?)? == n {
                    return;
                }
                self.set_current_animation(n)?;
                self.emit_state(patch, osc_controller)?;
            }
        }
    }

    fn current_animation_with_index<'a>(
        &self,
        patch: &'a mut Patch,
    ) -> Result<(&mut dyn ControllableTargetedAnimation, usize)> {
        let key = self.current_group()?;
        let animation_index = self.animation_index_for_key(key)?;
        let group = patch
            .group_mut(key)
            .ok_or_else(|| anyhow!("no group found for {key:?}"))?;
        Ok((
            &mut group
                .animations_mut()
                .ok_or_else(|| anyhow!("{key:?} does not have animations"))?[animation_index],
            animation_index,
        ))
    }

    fn current_animation<'a>(
        &self,
        patch: &'a mut Patch,
    ) -> Result<&mut dyn ControllableTargetedAnimation> {
        let (ta, _) = self.current_animation_with_index(patch)?;
        Ok(ta)
    }

    fn current_group(&self) -> Result<&FixtureGroupKey> {
        let group = self
            .current_group
            .as_ref()
            .ok_or_else(|| anyhow!("no animation group is selected"))?;
        Ok(group)
    }

    fn animation_index_for_key(&self, key: &FixtureGroupKey) -> Result<usize> {
        let index = self
            .selected_animator_by_group
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow!("no current animation set for {key:?}"))?;
        Ok(index)
    }

    /// Set the current animation for the current group to the provided value.
    pub fn set_current_animation(&mut self, n: usize) -> Result<()> {
        if n > N_ANIM {
            bail!("animator index {n} out of range");
        }
        let group = self.current_group()?.clone();
        match self.selected_animator_by_group.get_mut(&group) {
            Some(selected_animation) => {
                *selected_animation = n;
            }
            None => {
                bail!("no selected animator state for {group:?}");
            }
        }
        Ok(())
    }
}

impl EmitAnimationStateChange for OscController {
    fn emit_animation_state_change(&mut self, sc: tunnels::animation::StateChange) {
        self.emit(FixtureStateChangeWithGroup {
            group: GroupName::none(),
            sc: FixtureStateChange::Animation(sc),
        });
    }
}

pub enum ControlMessage {
    Animation(tunnels::animation::ControlMessage),
    Target(AnimationTargetIndex),
    Select(usize),
}

pub enum StateChange {
    Animation(tunnels::animation::StateChange),
    Target(AnimationTargetIndex),
    Select(usize),
}
