//! Roll up all possible fixture animation targets into one type.
use std::fmt::Debug;
use std::fmt::Display;

use anyhow::{bail, Result};
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use strum::IntoEnumIterator;
use tunnels::animation::Animation;

/// Numeric index for an animation target.
/// This is used to represent an animation target as a generic selection.
pub type AnimationTargetIndex = usize;

/// A collection of animation values paired with targets.
pub type TargetedAnimations<T> = [(f64, T)];

/// A pairing of an animation and a target.
#[derive(Debug, Clone, Default)]
pub struct TargetedAnimation<T: AnimationTarget> {
    pub animation: Animation,
    pub target: T,
}

pub trait AnimationTarget:
    ToPrimitive + FromPrimitive + IntoEnumIterator + Display + Clone + Copy + Default + Debug
{
}

impl<T> AnimationTarget for T where
    T: ToPrimitive + FromPrimitive + IntoEnumIterator + Display + Clone + Copy + Default + Debug
{
}

pub trait ControllableTargetedAnimation {
    fn anim(&self) -> &Animation;
    fn anim_mut(&mut self) -> &mut Animation;
    fn target(&self) -> AnimationTargetIndex;
    fn set_target(&mut self, index: AnimationTargetIndex) -> Result<()>;
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

    fn set_target(&mut self, index: AnimationTargetIndex) -> Result<()> {
        let Some(target) = T::from_usize(index) else {
            bail!("animation index {index} out of range for {}", std::any::type_name::<T>());
        };
        self.target = target;
        Ok(())
    }
}
