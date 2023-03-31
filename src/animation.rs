//! Maintain UI state for animations.

use simple_error::SimpleError;
use std::{collections::HashMap, error::Error};
use tunnels::animation::EmitStateChange as EmitAnimationStateChange;

use crate::{
    fixture::{
        animation_target::TargetedAnimation, EmitStateChange, FixtureGroupKey, FixtureStateChange,
        GroupName, Patch, StateChange,
    },
    osc::OscController,
};

#[derive(Default)]
pub struct AnimationUIState {
    pub current_group: Option<FixtureGroupKey>,
    pub selected_animator_by_group: HashMap<FixtureGroupKey, usize>,
}

impl AnimationUIState {
    pub fn current_animation<'a>(
        &self,
        patch: &'a mut Patch,
    ) -> Result<&'a mut TargetedAnimation, Box<dyn Error>> {
        let key = self
            .current_group
            .as_ref()
            .ok_or_else(|| SimpleError::new("no animation group is selected"))?;
        let animation_index = self
            .selected_animator_by_group
            .get(key)
            .ok_or_else(|| SimpleError::new(format!("no current animatior set for {key:?}")))?;
        let group = patch
            .group_mut(key)
            .ok_or_else(|| SimpleError::new(format!("no group found for {key:?}")))?;
        Ok(&mut group
            .animations_mut()
            .ok_or_else(|| SimpleError::new(format!("{key:?} does not have animations")))?
            [*animation_index])
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
