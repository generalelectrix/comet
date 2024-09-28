use anyhow::{ensure, Context, Result};
use fixture::FixtureControlMessage;
use itertools::Itertools;
use std::any::{type_name, Any};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail};
use lazy_static::lazy_static;
use log::{debug, info};
use number::{Phase, UnipolarFloat};
use serde::{Deserialize, Serialize};

use self::animation_target::{
    ControllableTargetedAnimation, TargetedAnimation, TargetedAnimationValues,
};
use self::profile::aquarius::Aquarius;
use self::profile::astroscan::Astroscan;
use self::profile::color::Color;
use self::profile::colordynamic::Colordynamic;
use self::profile::comet::Comet;
use self::profile::dimmer::Dimmer;
use self::profile::faderboard::Faderboard;
use self::profile::freedom_fries::FreedomFries;
use self::profile::h2o::H2O;
use self::profile::hypnotic::Hypnotic;
use self::profile::lumasphere::Lumasphere;
use self::profile::radiance::Radiance;
use self::profile::rotosphere_q3::RotosphereQ3;
use self::profile::rush_wizard::RushWizard;
use self::profile::solar_system::SolarSystem;
use self::profile::starlight::Starlight;
use self::profile::swarmolon::Swarmolon;
use self::profile::uv_led_brick::UvLedBrick;
use self::profile::venus::Venus;
use self::profile::wizard_extreme::WizardExtreme;
use crate::animation::ControlMessage as AnimationControlMessage;
use crate::config::{FixtureConfig, Options};
use crate::dmx::{DmxBuffer, UniverseIdx};
use crate::fixture::animation_target::AnimationTarget;
use crate::master::{ControlMessage as MasterControlMessage, MasterControls, Strobe};
use crate::osc::{MapControls, OscClientId, OscMessageWithGroupSender, TalkbackMode};
use crate::show::{ChannelId, ControlMessage as ShowControlMessage};

pub mod animation_target;
mod fixture;
mod group;
mod patch;
pub mod profile;

pub use fixture::FixtureType;
pub use group::{FixtureGroup, FixtureGroupKey, GroupName};
pub use patch::{Patch, PatchAnimatedFixture, PatchFixture};
pub use profile::*;

#[derive(Debug)]
pub struct ControlMessage {
    pub sender_id: OscClientId,
    pub talkback: TalkbackMode,
    // FIXME: this should be tied to the fixture message payload, not this scope!
    pub key: Option<FixtureGroupKey>,
    pub msg: ControlMessagePayload,
}

#[derive(Debug)]
pub enum ControlMessagePayload {
    Fixture(FixtureControlMessage),
    Master(MasterControlMessage),
    RefreshUI,
    Animation(AnimationControlMessage),
    Show(ShowControlMessage),
}

impl ControlMessagePayload {
    pub fn fixture<T: Any>(msg: T) -> Self {
        Self::Fixture(FixtureControlMessage(Box::new(msg)))
    }
}

pub const N_ANIM: usize = 4;
pub type TargetedAnimations<T> = [TargetedAnimation<T>; N_ANIM];

/// Wrap up the master and group-level controls into a single struct to pass
/// into fixtures.
pub struct FixtureGroupControls<'a> {
    /// Master controls.
    master_controls: &'a MasterControls,
    /// True if the fixture should render in mirrored mode.
    mirror: bool,
}

impl<'a> FixtureGroupControls<'a> {
    pub fn strobe(&self) -> &Strobe {
        self.master_controls.strobe()
    }
}

pub mod prelude {
    pub use super::fixture::{
        AnimatedFixture, ControllableFixture, FixtureControlMessage, FixtureType,
        NonAnimatedFixture,
    };
    pub use super::patch::{PatchAnimatedFixture, PatchFixture};
    #[allow(unused)]
    pub use super::FixtureGroupControls;
    pub use crate::fixture::animation_target::TargetedAnimationValues;
    pub use crate::osc::HandleStateChange;
}
