//! Roll up all possible fixture animation targets into one type.
use tunnels::animation::Animation;

use super::wizard_extreme::AnimationTarget as WizardExtremeAnimationTarget;

#[derive(Clone, Copy, Debug)]
pub enum AnimationTarget {
    None,
    WizardExtreme(WizardExtremeAnimationTarget),
}

/// A collection of animation values paired with targets.
pub type TargetedAnimations = [(f64, AnimationTarget)];

/// A pairing of an animation and a target.
#[derive(Debug, Clone)]
pub struct TargetedAnimation {
    pub animation: Animation,
    pub target: AnimationTarget,
}
