use anyhow::{ensure, Result};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

use anyhow::bail;
use lazy_static::lazy_static;
use log::info;

use super::fixture::{
    AnimatedFixture, Fixture, FixtureType, FixtureWithAnimations, NonAnimatedFixture,
};
use super::group::{FixtureGroup, FixtureGroupKey};
use super::profile::aquarius::Aquarius;
use super::profile::astroscan::Astroscan;
use super::profile::color::Color;
use super::profile::colordynamic::Colordynamic;
use super::profile::comet::Comet;
use super::profile::dimmer::Dimmer;
use super::profile::faderboard::Faderboard;
use super::profile::freedom_fries::FreedomFries;
use super::profile::h2o::H2O;
use super::profile::hypnotic::Hypnotic;
use super::profile::lumasphere::Lumasphere;
use super::profile::radiance::Radiance;
use super::profile::rotosphere_q3::RotosphereQ3;
use super::profile::rush_wizard::RushWizard;
use super::profile::solar_system::SolarSystem;
use super::profile::starlight::Starlight;
use super::profile::swarmolon::Swarmolon;
use super::profile::uv_led_brick::UvLedBrick;
use super::profile::venus::Venus;
use super::profile::wizard_extreme::WizardExtreme;
use crate::config::{FixtureConfig, Options};
use crate::dmx::UniverseIdx;
use crate::fixture::group::GroupFixtureConfig;
use crate::show::ChannelId;

type UsedAddrs = HashMap<(UniverseIdx, usize), FixtureConfig>;

#[derive(Default)]
pub struct Patch {
    fixtures: HashMap<FixtureGroupKey, FixtureGroup>,
    used_addrs: UsedAddrs,
    // Lookup from channel index to the fixture group assigned to that channel.
    channel_index: Vec<FixtureGroupKey>,
}

lazy_static! {
    static ref PATCHERS: Vec<Patcher> = vec![
        Astroscan::patcher(),
        Aquarius::patcher(),
        Color::patcher(),
        Colordynamic::patcher(),
        Comet::patcher(),
        Dimmer::patcher(),
        Faderboard::patcher(),
        FreedomFries::patcher(),
        H2O::patcher(),
        Hypnotic::patcher(),
        Lumasphere::patcher(),
        Radiance::patcher(),
        RotosphereQ3::patcher(),
        RushWizard::patcher(),
        SolarSystem::patcher(),
        Swarmolon::patcher(),
        Starlight::patcher(),
        UvLedBrick::patcher(),
        Venus::patcher(),
        WizardExtreme::patcher(),
    ];
}

impl Patch {
    pub fn patch(&mut self, cfg: FixtureConfig) -> anyhow::Result<()> {
        let mut candidates = PATCHERS
            .iter()
            .flat_map(|p| p(&cfg))
            .collect::<Result<Vec<_>>>()?;
        let candidate = match candidates.len() {
            0 => bail!("unable to patch {cfg:?}"),
            1 => candidates.pop().unwrap(),
            _ => bail!(
                "multiple fixture patch candidates: {:?}",
                candidates.iter().map(|c| &c.fixture_type).join(", ")
            ),
        };
        self.used_addrs = self.check_collision(&candidate, &cfg)?;
        // Add channel mapping index if provided.  Ensure this is an animatable fixture.
        if cfg.channel {
            ensure!(
                candidate.fixture.is_animated(),
                "cannot assign non-animatable fixture {} to a channel",
                candidate.fixture_type
            );
        }
        info!(
            "Controlling {} at {} (group: {}).",
            cfg.name,
            cfg.addr,
            cfg.group.as_deref().unwrap_or("none")
        );
        let key = FixtureGroupKey {
            fixture: candidate.fixture_type,
            group: cfg.group,
        };
        // Either identify an existing appropriate group or create a new one.
        if let Some(group) = self.fixtures.get_mut(&key) {
            group.patch(GroupFixtureConfig {
                universe: cfg.universe,
                dmx_addr: cfg.addr.dmx_index(),
                mirror: cfg.mirror,
            });
            return Ok(());
        }
        // No existing group; create a new one.
        if cfg.channel {
            self.channel_index.push(key.clone());
        }
        self.fixtures.insert(
            key.clone(),
            FixtureGroup::new(
                key,
                GroupFixtureConfig {
                    universe: cfg.universe,
                    dmx_addr: cfg.addr.dmx_index(),
                    mirror: cfg.mirror,
                },
                candidate.channel_count,
                candidate.fixture,
            ),
        );

        Ok(())
    }

    /// Dynamically get the universe count.
    pub fn universe_count(&self) -> usize {
        let mut universes = HashSet::new();
        for group in self.fixtures.values() {
            for element in group.fixture_configs() {
                universes.insert(element.universe);
            }
        }
        universes.len()
    }

    /// Check that the patch candidate doesn't conflict with another patched fixture.
    /// Return an updated collection of used addresses if it does not conflict.
    fn check_collision(
        &self,
        candidate: &PatchCandidate,
        cfg: &FixtureConfig,
    ) -> Result<UsedAddrs> {
        let mut used_addrs = self.used_addrs.clone();
        let dmx_index = cfg.addr.dmx_index();
        for addr in dmx_index..dmx_index + candidate.channel_count {
            match used_addrs.get(&(cfg.universe, addr)) {
                Some(existing_fixture) => {
                    bail!(
                        "{} at {} overlaps at DMX address {} in universe {} with {} at {}.",
                        cfg.name,
                        cfg.addr,
                        addr + 1,
                        cfg.universe,
                        existing_fixture.name,
                        existing_fixture.addr,
                    );
                }
                None => {
                    used_addrs.insert((cfg.universe, addr), cfg.clone());
                }
            }
        }
        Ok(used_addrs)
    }

    /// Get a fixture group by channel ID.
    pub fn group_by_channel_mut(&mut self, channel: ChannelId) -> Result<&mut FixtureGroup> {
        let Some(fixture_key) = self.channel_index.get(channel.0) else {
            bail!("tried to get out-of-range channel {}.", channel.0);
        };
        if let Some(fixture) = self.fixtures.get_mut(fixture_key) {
            Ok(fixture)
        } else {
            bail!(
                "channel ID {} mapped to non-existent fixture key {fixture_key}",
                channel.0
            );
        }
    }

    /// Validate that a channel index refers to a channel that actually exists.
    pub fn validate_channel(&self, channel: usize) -> Result<ChannelId> {
        if channel < self.channel_index.len() {
            Ok(ChannelId(channel))
        } else {
            bail!(
                "channel selector {channel} out of range, only {} channels are configured",
                self.channel_index.len()
            );
        }
    }

    /// Iterate over all of the labels for each channels.
    pub fn channel_labels(&self) -> impl Iterator<Item = String> + '_ {
        self.channel_index
            .iter()
            .filter_map(|i| self.fixtures.get(i))
            .map(|f| match f.name() {
                None => f.fixture_type().to_string(),
                Some(g) => format!("{g}({})", f.fixture_type()),
            })
    }

    /// Get the fixture patched with this key, mutably.
    pub fn get_mut(&mut self, key: &FixtureGroupKey) -> Option<&mut FixtureGroup> {
        self.fixtures.get_mut(key)
    }

    /// Iterate over all patched fixtures.
    pub fn iter(&self) -> impl Iterator<Item = &FixtureGroup> {
        self.fixtures.values()
    }

    /// Iterate over all patched fixtures, mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut FixtureGroup> {
        self.fixtures.values_mut()
    }
}

pub struct PatchCandidate {
    fixture_type: FixtureType,
    channel_count: usize,
    fixture: Box<dyn Fixture>,
}

pub type Patcher = Box<dyn Fn(&FixtureConfig) -> Option<Result<PatchCandidate>> + Sync>;

/// Fixture constructor trait to handle patching non-animating fixtures.
pub trait PatchFixture: NonAnimatedFixture + Default + 'static {
    const NAME: FixtureType;

    /// Return a closure that will try to patch a fixture if it has the appropriate name.
    fn patcher() -> Patcher {
        Box::new(|cfg| {
            if *cfg.name != *Self::NAME {
                return None;
            }
            match Self::new(&cfg.options) {
                Ok(fixture) => Some(Ok(PatchCandidate {
                    fixture_type: Self::NAME,
                    channel_count: fixture.channel_count(),
                    fixture: Box::new(fixture),
                })),
                Err(e) => Some(Err(e)),
            }
        })
    }

    /// The number of contiguous DMX channels used by the fixture.
    fn channel_count(&self) -> usize;

    /// Create a new instance of the fixture from the provided options.
    /// Non-customizable fixtures will fall back to using default.
    /// This can be overridden for fixtures that are customizable.
    fn new(_options: &Options) -> Result<Self> {
        Ok(Self::default())
    }
}

/// Fixture constructor trait to handle patching non-animating fixtures.
pub trait PatchAnimatedFixture: AnimatedFixture + Default + 'static {
    const NAME: FixtureType;

    /// Return a closure that will try to patch a fixture if it has the appropriate name.
    fn patcher() -> Patcher {
        Box::new(|cfg| {
            if *cfg.name != *Self::NAME {
                return None;
            }
            match Self::new(&cfg.options) {
                Ok(fixture) => Some(Ok(PatchCandidate {
                    fixture_type: Self::NAME,
                    channel_count: fixture.channel_count(),
                    fixture: Box::new(FixtureWithAnimations {
                        fixture,
                        animations: Default::default(),
                    }),
                })),
                Err(e) => Some(Err(e)),
            }
        })
    }

    /// The number of contiguous DMX channels used by the fixture.
    fn channel_count(&self) -> usize;

    /// Create a new instance of the fixture from the provided options.
    /// Non-customizable fixtures will fall back to using default.
    /// This can be overridden for fixtures that are customizable.
    fn new(_options: &Options) -> Result<Self> {
        Ok(Self::default())
    }
}
