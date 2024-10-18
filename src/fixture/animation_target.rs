use std::fmt::Debug;
use std::fmt::Display;

use anyhow::bail;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use strum::IntoEnumIterator;
use tunnels::animation::Animation;

pub const N_ANIM: usize = 4;
pub type TargetedAnimations<T> = [TargetedAnimation<T>; N_ANIM];

/// Numeric index for an animation target.
/// This is used to represent an animation target as a generic selection.
pub type AnimationTargetIndex = usize;

/// A collection of animation values paired with targets.
pub type TargetedAnimationValues<T> = [(f64, T)];

/// A pairing of an animation and a target.
#[derive(Debug, Clone, Default)]
pub struct TargetedAnimation<T: AnimationTarget> {
    pub animation: Animation,
    pub target: T,
}

/// An animation target should be an enum with a unit variant for each option.
pub trait AnimationTarget:
    ToPrimitive + FromPrimitive + IntoEnumIterator + Display + Clone + Copy + Default + Debug
{
}

impl<T> AnimationTarget for T where
    T: ToPrimitive + FromPrimitive + IntoEnumIterator + Display + Clone + Copy + Default + Debug
{
}

/// Interface to a targeted animation.
/// Targets are handled as numeric indices.
pub trait ControllableTargetedAnimation {
    /// Get an immutable reference to the inner animation.
    fn anim(&self) -> &Animation;
    /// Get a mutable reference to the inner animation.
    fn anim_mut(&mut self) -> &mut Animation;
    /// Get the current animation target as an index.
    fn target(&self) -> AnimationTargetIndex;
    /// Set the current animation target to the provided index.
    /// Return an error if the index is invalid for this target type.
    fn set_target(&mut self, index: AnimationTargetIndex) -> anyhow::Result<()>;
    /// Return the labels for the animation target type.
    fn target_labels(&self) -> Vec<String>;
}

impl<T: AnimationTarget> ControllableTargetedAnimation for TargetedAnimation<T> {
    fn anim(&self) -> &Animation {
        &self.animation
    }

    fn anim_mut(&mut self) -> &mut Animation {
        &mut self.animation
    }

    fn target(&self) -> AnimationTargetIndex {
        self.target.to_usize().unwrap()
    }

    fn set_target(&mut self, index: AnimationTargetIndex) -> anyhow::Result<()> {
        let Some(target) = T::from_usize(index) else {
            bail!(
                "animation index {index} out of range for {}",
                std::any::type_name::<T>()
            );
        };
        self.target = target;
        Ok(())
    }

    fn target_labels(&self) -> Vec<String> {
        T::iter().map(|t| t.to_string()).collect()
    }
}
