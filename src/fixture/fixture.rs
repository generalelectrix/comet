//! Types related to specifying and controlling individual fixture models.
use anyhow::Result;
use std::any::{type_name, Any};
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::time::Duration;

use anyhow::anyhow;
use number::{Phase, UnipolarFloat};
use serde::{Deserialize, Serialize};

use super::animation_target::{
    ControllableTargetedAnimation, TargetedAnimationValues, TargetedAnimations, N_ANIM,
};
use super::{ControlMessagePayload, FixtureGroupControls};
use crate::channel::{ChannelControlMessage, ChannelStateEmitter};
use crate::fixture::animation_target::AnimationTarget;
use crate::osc::MapControls;

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

#[derive(Debug)]
pub struct OwnedFixtureControlMessage(pub Box<dyn Any>);

impl OwnedFixtureControlMessage {
    pub fn borrowed(&self) -> FixtureControlMessage<'_> {
        FixtureControlMessage(self.0.as_ref())
    }
}

pub struct FixtureControlMessage<'a>(&'a dyn Any);

impl<'a> FixtureControlMessage<'a> {
    pub fn unpack_as<T: 'static>(&self) -> Result<&'a T> {
        self.0
            .downcast_ref()
            .ok_or_else(|| anyhow!("could not unpack message as {}", type_name::<T>()))
    }
}

pub trait ControllableFixture: MapControls {
    /// Emit the current state of all controls.
    fn emit_state(&self, emitter: &dyn crate::osc::EmitControlMessage);

    /// Emit the current state of all controls that are bound to channel-level controls.
    #[allow(unused)]
    fn emit_state_for_channel(&self, emitter: &ChannelStateEmitter) {}

    /// Process the provided control message.
    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &dyn crate::osc::EmitControlMessage,
    ) -> anyhow::Result<()>;

    /// Process a channel control message, if the fixture uses it.
    #[allow(unused)]
    fn control_from_channel(
        &mut self,
        msg: &ChannelControlMessage,
        emitter: &dyn crate::osc::EmitControlMessage,
    ) {
        // Ignore channel control messages by default.
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
        animation_vals: &TargetedAnimationValues<Self::Target>,
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
    fn get_animation(&mut self, index: usize) -> Option<&mut dyn ControllableTargetedAnimation>;
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

    fn get_animation(&mut self, _index: usize) -> Option<&mut dyn ControllableTargetedAnimation> {
        None
    }
}

#[derive(Debug)]
pub struct FixtureWithAnimations<F: AnimatedFixture> {
    pub fixture: F,
    pub animations: TargetedAnimations<F::Target>,
}

impl<F: AnimatedFixture> MapControls for FixtureWithAnimations<F> {
    fn map_controls(&self, map: &mut crate::osc::ControlMap<ControlMessagePayload>) {
        self.fixture.map_controls(map)
    }

    fn fixture_type_aliases(&self) -> Vec<(String, FixtureType)> {
        self.fixture.fixture_type_aliases()
    }
}

impl<F: AnimatedFixture> ControllableFixture for FixtureWithAnimations<F> {
    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &dyn crate::osc::EmitControlMessage,
    ) -> anyhow::Result<()> {
        self.fixture.control(msg, emitter)
    }

    fn emit_state(&self, emitter: &dyn crate::osc::EmitControlMessage) {
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
        self.fixture
            .render_with_animations(group_controls, &animation_vals, dmx_buffer);
    }

    fn is_animated(&self) -> bool {
        true
    }

    fn get_animation(&mut self, index: usize) -> Option<&mut dyn ControllableTargetedAnimation> {
        let animation = self.animations.get_mut(index)?;
        Some(&mut *animation)
    }
}
