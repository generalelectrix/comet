//! Maintain UI state for animations.

use std::collections::HashMap;
use tunnels::animation::EmitStateChange as EmitAnimationStateChange;

use crate::{
    fixture::{EmitStateChange, FixtureGroupKey, FixtureStateChange, GroupName, StateChange},
    osc::OscController,
};

#[derive(Default)]
pub struct AnimationUIState {
    pub current_group: Option<FixtureGroupKey>,
    pub selected_animator_by_group: HashMap<FixtureGroupKey, usize>,
}

impl EmitAnimationStateChange for OscController {
    fn emit_animation_state_change(&mut self, sc: tunnels::animation::StateChange) {
        self.emit(StateChange {
            group: GroupName::none(),
            sc: FixtureStateChange::Animation(sc),
        });
    }
}
