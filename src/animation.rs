//! Maintain UI state for animations.

use simple_error::{bail, SimpleError};
use std::{collections::HashMap, error::Error};
use tunnels::animation::EmitStateChange as EmitAnimationStateChange;

use crate::{
    fixture::{
        animation_target::TargetedAnimation, EmitStateChange, FixtureControlMessage,
        FixtureGroupKey, FixtureStateChange, GroupName, Patch, StateChange, N_ANIM,
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
    ) -> Result<(), Box<dyn Error>> {
        let (ta, index) = self.current_animation_with_index(patch)?;
        ta.animation.emit_state(osc_controller);
        osc_controller.emit(StateChange {
            group: GroupName::none(),
            sc: FixtureStateChange::AnimationTarget(ta.target),
        });
        osc_controller.emit(StateChange {
            group: GroupName::none(),
            sc: FixtureStateChange::AnimationSelect(index),
        });
        Ok(())
    }

    /// Handle a control message.
    pub fn control(
        &mut self,
        msg: FixtureControlMessage,
        patch: &mut Patch,
        osc_controller: &mut OscController,
    ) -> Result<(), Box<dyn Error>> {
        match msg {
            FixtureControlMessage::Animation(msg) => {
                self.current_animation(patch)?
                    .animation
                    .control(msg, osc_controller);
            }
            FixtureControlMessage::AnimationTarget(msg) => {
                self.current_animation(patch)?.target = msg;
                osc_controller.emit(StateChange {
                    group: GroupName::none(),
                    sc: crate::fixture::FixtureStateChange::AnimationTarget(msg),
                });
            }
            FixtureControlMessage::AnimationSelect(n) => {
                if self.animation_index_for_key(self.current_group()?)? == n {
                    return Ok(());
                }
                self.set_current_animation(n)?;
                self.emit_state(patch, osc_controller)?;
            }
            _ => bail!("FIXME make this impossible: unexpected control message passed to animation UI: {msg:?}")
        }

        Ok(())
    }

    fn current_animation_with_index<'a>(
        &self,
        patch: &'a mut Patch,
    ) -> Result<(&'a mut TargetedAnimation, usize), Box<dyn Error>> {
        let key = self.current_group()?;
        let animation_index = self.animation_index_for_key(key)?;
        let group = patch
            .group_mut(key)
            .ok_or_else(|| SimpleError::new(format!("no group found for {key:?}")))?;
        Ok((
            &mut group
                .animations_mut()
                .ok_or_else(|| SimpleError::new(format!("{key:?} does not have animations")))?
                [animation_index],
            animation_index,
        ))
    }

    fn current_animation<'a>(
        &self,
        patch: &'a mut Patch,
    ) -> Result<&'a mut TargetedAnimation, Box<dyn Error>> {
        let (ta, _) = self.current_animation_with_index(patch)?;
        Ok(ta)
    }

    fn current_group(&self) -> Result<&FixtureGroupKey, Box<dyn Error>> {
        let group = self
            .current_group
            .as_ref()
            .ok_or_else(|| SimpleError::new("no animation group is selected"))?;
        Ok(group)
    }

    fn animation_index_for_key(&self, key: &FixtureGroupKey) -> Result<usize, Box<dyn Error>> {
        let index = self
            .selected_animator_by_group
            .get(key)
            .cloned()
            .ok_or_else(|| SimpleError::new(format!("no current animation set for {key:?}")))?;
        Ok(index)
    }

    /// Set the current animation for the current group to the provided value.
    pub fn set_current_animation(&mut self, n: usize) -> Result<(), Box<dyn Error>> {
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
        self.emit(StateChange {
            group: GroupName::none(),
            sc: FixtureStateChange::Animation(sc),
        });
    }
}
