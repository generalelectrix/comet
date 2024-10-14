//! Define groups of fixtures, sharing a common fixture

use anyhow::Context;
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use log::debug;
use number::{Phase, UnipolarFloat};
use serde::{Deserialize, Serialize};

use super::animation_target::ControllableTargetedAnimation;
use super::fixture::{Fixture, FixtureControlMessage, FixtureType};
use super::prelude::ChannelStateEmitter;
use super::ControlMessagePayload;
use crate::channel::ChannelControlMessage;
use crate::dmx::DmxBuffer;
use crate::master::{FixtureGroupControls, MasterControls};
use crate::osc::{MapControls, OscMessageWithGroupSender};

#[derive(Debug)]
pub struct FixtureGroup {
    /// The unique identifier of this group.
    key: FixtureGroupKey,
    /// The configurations for the fixtures in the group.
    fixture_configs: Vec<GroupFixtureConfig>,
    /// The number of DMX channels used by this fixture.
    channel_count: usize,
    /// The inner implementation of the fixture.
    fixture: Box<dyn Fixture>,
}

impl FixtureGroup {
    /// Create a fixture group, containing a single fixture config.
    pub fn new(
        key: FixtureGroupKey,
        fixture_config: GroupFixtureConfig,
        channel_count: usize,
        fixture: Box<dyn Fixture>,
    ) -> Self {
        Self {
            key,
            fixture_configs: vec![fixture_config],
            channel_count,
            fixture,
        }
    }

    /// Patch an additional fixture in this group.
    pub fn patch(&mut self, cfg: GroupFixtureConfig) {
        self.fixture_configs.push(cfg);
    }

    pub fn key(&self) -> &FixtureGroupKey {
        &self.key
    }
    pub fn fixture_type(&self) -> &FixtureType {
        &self.key.fixture
    }

    pub fn name(&self) -> Option<&GroupName> {
        self.key.group.as_ref()
    }

    pub fn get_animation(
        &mut self,
        index: usize,
    ) -> Option<&mut dyn ControllableTargetedAnimation> {
        self.fixture.get_animation(index)
    }

    pub fn is_animated(&self) -> bool {
        self.fixture.is_animated()
    }

    pub fn fixture_configs(&self) -> &[GroupFixtureConfig] {
        &self.fixture_configs
    }

    /// Emit the current state of all controls.
    pub fn emit_state(&self, emitter: &dyn crate::osc::EmitControlMessage) {
        let mut emitter = OscMessageWithGroupSender {
            group: self.name(),
            emitter,
        };
        self.fixture.emit_state(&emitter);
    }

    /// Emit the current state of all controls bound to channel-level controls.
    pub fn emit_state_for_channel(&self, emitter: &ChannelStateEmitter) {
        self.fixture.emit_state_for_channel(emitter);
    }

    /// Process the provided control message.
    pub fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &dyn crate::osc::EmitControlMessage,
    ) -> anyhow::Result<()> {
        let emitter = OscMessageWithGroupSender {
            group: self.key.group.as_ref(),
            emitter,
        };
        self.fixture
            .control(msg, &emitter)
            .with_context(|| self.key.clone())
    }

    /// Process the provided channel control message.
    pub fn control_from_channel(
        &mut self,
        msg: &ChannelControlMessage,
        emitter: &dyn crate::osc::EmitControlMessage,
    ) {
        let emitter = OscMessageWithGroupSender {
            group: self.key.group.as_ref(),
            emitter,
        };
        self.fixture.control_from_channel(msg, &emitter);
    }

    pub fn update(&mut self, delta_t: Duration, _audio_envelope: UnipolarFloat) {
        self.fixture.update(delta_t);
    }

    /// Render into the provided DMX universe.
    /// The master controls are provided to potentially alter the render.
    pub fn render(&self, master_controls: &MasterControls, dmx_buffers: &mut [DmxBuffer]) {
        let phase_offset_per_fixture = Phase::new(1.0 / self.fixture_configs.len() as f64);
        for (i, cfg) in self.fixture_configs.iter().enumerate() {
            let phase_offset = phase_offset_per_fixture * i as f64;
            let dmx_buf =
                &mut dmx_buffers[cfg.universe][cfg.dmx_addr..cfg.dmx_addr + self.channel_count];
            self.fixture.render(
                phase_offset,
                &FixtureGroupControls {
                    master_controls,
                    mirror: cfg.mirror,
                },
                dmx_buf,
            );
            debug!(
                "{} ({}): {:?}",
                self.fixture_type(),
                self.name().map(|g| g.0.as_str()).unwrap_or("none"),
                dmx_buf
            );
        }
    }
}

#[derive(Debug)]
pub struct GroupFixtureConfig {
    /// The starting index into the DMX buffer for a fixture in a group.
    pub dmx_addr: usize,
    /// The universe that this fixture is patched in.
    pub universe: usize,
    /// True if the fixture should be mirrored in mirror mode.
    pub mirror: bool,
}

impl MapControls for FixtureGroup {
    fn map_controls(&self, map: &mut crate::osc::ControlMap<ControlMessagePayload>) {
        self.fixture.map_controls(map);
    }

    fn fixture_type_aliases(&self) -> Vec<(String, FixtureType)> {
        self.fixture.fixture_type_aliases()
    }
}

/// Uniquely identify a specific fixture group.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FixtureGroupKey {
    pub fixture: FixtureType,
    pub group: Option<GroupName>,
}

impl Display for FixtureGroupKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.group.as_ref().map(|g| g.0.as_str()).unwrap_or("none"),
            self.fixture
        )
    }
}

/// User-provided name for a particular fixture group.
#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct GroupName(Arc<String>);

impl GroupName {
    pub fn new<S: Into<String>>(v: S) -> Self {
        Self(Arc::new(v.into()))
    }
}

impl Deref for GroupName {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl Display for GroupName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
