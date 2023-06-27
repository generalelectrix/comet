//! Maintain UI state for animations.
use anyhow::{anyhow, bail, Result};
use serde::Deserialize;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct GroupSelection(pub usize);

#[derive(Default)]
pub struct AnimationUIState {
    pub current_group: Option<GroupSelection>,
    pub selected_animator_by_group: HashMap<GroupSelection, usize>,
}

impl AnimationUIState {
    /// Emit all current animation state, including target and selection.
    pub fn emit_state(
        &self,
        patch: &mut Patch,                  // FIXME this doesn't need to be mutable
        osc_controller: &mut OscController, // FIXME this ought to be generic
    ) -> Result<()> {
        let (ta, index) = self.current_animation_with_index(patch)?;
        ta.anim().emit_state(osc_controller);
        osc_controller.emit(FixtureStateChangeWithGroup {
            group: GroupName::none(),
            sc: FixtureStateChange::Animation(StateChange::Target(ta.target())),
        });
        osc_controller.emit(FixtureStateChangeWithGroup {
            group: GroupName::none(),
            sc: FixtureStateChange::Animation(StateChange::SelectAnimation(index)),
        });
        osc_controller.emit(FixtureStateChangeWithGroup {
            group: GroupName::none(),
            sc: FixtureStateChange::Animation(StateChange::TargetLabels(ta.target_labels())),
        });
        // FIXME this really should belong to the show
        osc_controller.emit(FixtureStateChangeWithGroup {
            group: GroupName::none(),
            sc: FixtureStateChange::Animation(StateChange::GroupLabels(patch.selector_labels())),
        });
        Ok(())
    }

    /// Handle a control message.
    pub fn control(
        &mut self,
        msg: ControlMessage,
        patch: &mut Patch,
        osc_controller: &mut OscController,
    ) -> Result<()> {
        match msg {
            ControlMessage::Animation(msg) => {
                self.current_animation(patch)?
                    .anim_mut()
                    .control(msg, osc_controller);
            }
            ControlMessage::Target(msg) => {
                let anim = self.current_animation(patch)?;
                if anim.target() == msg {
                    return Ok(());
                }
                anim.set_target(msg)?;
                osc_controller.emit(FixtureStateChangeWithGroup {
                    group: GroupName::none(),
                    sc: crate::fixture::FixtureStateChange::Animation(StateChange::Target(msg)),
                });
            }
            ControlMessage::SelectAnimation(n) => {
                if self.animation_index_for_selector(self.current_group()?)? == n {
                    return Ok(());
                }
                self.set_current_animation(n)?;
                self.emit_state(patch, osc_controller)?;
            }
            ControlMessage::SelectGroup(g) => {
                if patch.group_by_selector_mut(&g).is_none() {
                    bail!("group selector {:?} is not defined", g.0);
                }
                self.current_group = Some(g);
                self.emit_state(patch, osc_controller)?;
            }
        }
        Ok(())
    }

    fn current_animation_with_index<'a>(
        &self,
        patch: &'a mut Patch,
    ) -> Result<(&'a mut dyn ControllableTargetedAnimation, usize)> {
        let selector = self.current_group()?;
        let animation_index = self.animation_index_for_selector(selector)?;
        let group = patch
            .group_by_selector_mut(selector)
            .ok_or_else(|| anyhow!("no group found for selector {selector:?}"))?;
        let key = group.key().clone();
        if let Some(anim) = group.get_animation(animation_index) {
            return Ok((anim, animation_index));
        }
        bail!("{key:?} does not have animations");
    }

    fn current_animation<'a>(
        &self,
        patch: &'a mut Patch,
    ) -> Result<&'a mut dyn ControllableTargetedAnimation> {
        let (ta, _) = self.current_animation_with_index(patch)?;
        Ok(ta)
    }

    fn current_group(&self) -> Result<&GroupSelection> {
        let group = self
            .current_group
            .as_ref()
            .ok_or_else(|| anyhow!("no animation group is selected"))?;
        Ok(group)
    }

    fn animation_index_for_selector(&self, key: &GroupSelection) -> Result<usize> {
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
        let group = *self.current_group()?;
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
            sc: FixtureStateChange::Animation(StateChange::Animation(sc)),
        });
    }
}

#[derive(Clone, Debug)]
pub enum ControlMessage {
    Animation(tunnels::animation::ControlMessage),
    Target(AnimationTargetIndex),
    SelectAnimation(usize),
    SelectGroup(GroupSelection),
}

#[derive(Clone, Debug)]
pub enum StateChange {
    Animation(tunnels::animation::StateChange),
    Target(AnimationTargetIndex),
    SelectAnimation(usize),
    SelectGroup(GroupSelection),
    TargetLabels(Vec<String>),
    GroupLabels(Vec<String>),
}
