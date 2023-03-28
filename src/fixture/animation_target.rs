//! Roll up all possible fixture animation targets into one type.
use super::wizard_extreme::AnimationTarget as WizardExtremeAnimationTarget;

#[derive(Clone, Copy)]
pub enum AnimationTarget {
    WizardExtreme(WizardExtremeAnimationTarget),
}

/// A collection of animation values paired with targets.
pub type TargetedAnimations = Vec<(f64, AnimationTarget)>;
