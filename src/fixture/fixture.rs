//! Types related to specifying and controlling individual fixture models.
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::time::Duration;

use number::{Phase, UnipolarFloat};
use serde::{Deserialize, Serialize};

use super::animation_target::{
    ControllableTargetedAnimation, TargetedAnimationValues, TargetedAnimations, N_ANIM,
};
use super::FixtureGroupControls;
use crate::channel::ChannelControlMessage;
use crate::fixture::animation_target::AnimationTarget;
use crate::osc::{FixtureStateEmitter, OscControlMessage};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FixtureType(pub &'static str);

impl Deref for FixtureType {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Display for FixtureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

pub trait ControllableFixture {
    /// Emit the current state of all controls.
    fn emit_state(&self, emitter: &FixtureStateEmitter);

    /// Process the provided OSC control message.
    ///
    /// Return true if the control message was handled.
    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool>;

    /// Process a channel control message, if the fixture uses it.
    ///
    #[allow(unused)]
    fn control_from_channel(
        &mut self,
        msg: &ChannelControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        // Ignore channel control messages by default.
        Ok(())
    }

    fn update(&mut self, _: Duration) {}
}

pub trait NonAnimatedFixture: ControllableFixture + Debug {
    /// Render into the provided DMX buffer.
    /// The buffer will be pre-sized to the fixture's channel count and offset
    /// to the fixture's start address.
    /// The master controls are provided to potentially alter the render process.
    fn render(&self, group_controls: &FixtureGroupControls, dmx_buffer: &mut [u8]);
}

pub trait AnimatedFixture: ControllableFixture + Debug {
    type Target: AnimationTarget;

    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    );
}

pub trait Fixture: ControllableFixture + Debug {
    /// Render into the provided DMX buffer.
    /// The buffer will be pre-sized to the fixture's channel count and offset
    /// to the fixture's start address.
    /// The master controls are provided to potentially alter the render process.
    /// An animation phase offset is provided.
    fn render(
        &self,
        phase_offset: Phase,
        group_controls: &FixtureGroupControls,
        dmx_buffer: &mut [u8],
    );

    /// Return true if this fixture has animations.
    fn is_animated(&self) -> bool;

    /// Get the animation with the provided index.
    fn get_animation(&self, index: usize) -> Option<&dyn ControllableTargetedAnimation>;

    /// Get the animation with the provided index, mutably.
    fn get_animation_mut(&mut self, index: usize)
        -> Option<&mut dyn ControllableTargetedAnimation>;
}

impl<T> Fixture for T
where
    T: NonAnimatedFixture,
{
    fn render(
        &self,
        _phase_offset: Phase,
        group_controls: &FixtureGroupControls,
        dmx_buffer: &mut [u8],
    ) {
        self.render(group_controls, dmx_buffer)
    }

    fn is_animated(&self) -> bool {
        false
    }

    fn get_animation_mut(
        &mut self,
        _index: usize,
    ) -> Option<&mut dyn ControllableTargetedAnimation> {
        None
    }

    fn get_animation(&self, _index: usize) -> Option<&dyn ControllableTargetedAnimation> {
        None
    }
}

#[derive(Debug)]
pub struct FixtureWithAnimations<F: AnimatedFixture> {
    pub fixture: F,
    pub animations: TargetedAnimations<F::Target>,
}

impl<F: AnimatedFixture> ControllableFixture for FixtureWithAnimations<F> {
    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        self.fixture.control(msg, emitter)
    }

    fn control_from_channel(
        &mut self,
        msg: &ChannelControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        self.fixture.control_from_channel(msg, emitter)
    }

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.fixture.emit_state(emitter);
    }

    fn update(&mut self, dt: Duration) {
        self.fixture.update(dt);
        for ta in &mut self.animations {
            ta.animation.update_state(dt, UnipolarFloat::ZERO);
        }
    }
}

impl<F: AnimatedFixture> Fixture for FixtureWithAnimations<F> {
    fn render(
        &self,
        phase_offset: Phase,
        group_controls: &FixtureGroupControls,
        dmx_buffer: &mut [u8],
    ) {
        let mut animation_vals = [(0.0, F::Target::default()); N_ANIM];
        // FIXME: implement unipolar variant of animations
        for (i, ta) in self.animations.iter().enumerate() {
            animation_vals[i] = (
                ta.animation.get_value(
                    phase_offset,
                    &group_controls.master_controls.clock_state,
                    group_controls.master_controls.audio_envelope,
                ),
                ta.target,
            );
        }
        self.fixture.render_with_animations(
            group_controls,
            TargetedAnimationValues(&animation_vals),
            dmx_buffer,
        );
    }

    fn is_animated(&self) -> bool {
        true
    }

    fn get_animation_mut(
        &mut self,
        index: usize,
    ) -> Option<&mut dyn ControllableTargetedAnimation> {
        let animation = self.animations.get_mut(index)?;
        Some(&mut *animation)
    }

    fn get_animation(&self, index: usize) -> Option<&dyn ControllableTargetedAnimation> {
        let animation = self.animations.get(index)?;
        Some(animation)
    }
}
