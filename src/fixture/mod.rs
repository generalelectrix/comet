use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use std::time::Duration;

use log::{debug, info};
use number::{Phase, UnipolarFloat};
use serde::{Deserialize, Serialize};
use simple_error::bail;

use self::animation_target::{TargetedAnimation, TargetedAnimations};
use self::aquarius::{
    Aquarius, ControlMessage as AquariusControlMessage, StateChange as AquariusStateChange,
};
use self::color::{Color, ControlMessage as ColorControlMessage, StateChange as ColorStateChange};
use self::comet::{Comet, ControlMessage as CometControlMessage, StateChange as CometStateChange};
use self::dimmer::{
    ControlMessage as DimmerControlMessage, Dimmer, StateChange as DimmerStateChange,
};
use self::faderboard::{
    ControlMessage as FaderboardControlMessage, Faderboard, StateChange as FaderboardStateChange,
};
use self::freedom_fries::{
    ControlMessage as FreedomFriesControlMessage, FreedomFries,
    StateChange as FreedomFriesStateChange,
};
use self::h2o::{ControlMessage as H2OControlMessage, StateChange as H2OStateChange, H2O};
use self::lumasphere::{
    ControlMessage as LumasphereControlMessage, Lumasphere, StateChange as LumasphereStateChange,
};
use self::radiance::{
    ControlMessage as RadianceControlMessage, Radiance, StateChange as RadianceStateChange,
};
use self::rotosphere_q3::{
    ControlMessage as RotosphereQ3ControlMessage, RotosphereQ3,
    StateChange as RotosphereQ3StateChange,
};
use self::rush_wizard::{
    ControlMessage as RushWizardControlMessage, RushWizard, StateChange as RushWizardStateChange,
};
use self::swarmolon::{
    ControlMessage as SwarmolonControlMessage, StateChange as SwarmolonStateChange, Swarmolon,
};
use self::venus::{ControlMessage as VenusControlMessage, StateChange as VenusStateChange, Venus};
use self::wizard_extreme::{
    ControlMessage as WizardExtremeControlMessage, StateChange as WizardExtremeStateChange,
    WizardExtreme,
};
use crate::config::FixtureConfig;
use crate::fixture::animation_target::AnimationTarget;
use crate::master::{
    ControlMessage as MasterControlMessage, MasterControls, StateChange as MasterStateChange,
};
use crate::osc::MapControls;

pub mod animation_target;
pub mod aquarius;
pub mod color;
pub mod comet;
pub mod dimmer;
pub mod faderboard;
pub mod freedom_fries;
pub mod generic;
pub mod h2o;
pub mod lumasphere;
pub mod radiance;
pub mod rotosphere_q3;
pub mod rush_wizard;
pub mod swarmolon;
pub mod venus;
pub mod wizard_extreme;

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct GroupName(Option<Arc<String>>);

impl GroupName {
    pub fn none() -> Self {
        Self(None)
    }

    pub fn new<S: Into<String>>(v: S) -> Self {
        Self(Some(Arc::new(v.into())))
    }

    pub fn inner(&self) -> &Option<Arc<String>> {
        &self.0
    }
}

impl From<&Option<String>> for GroupName {
    fn from(v: &Option<String>) -> Self {
        match v {
            None => Self::none(),
            Some(v) => Self::new(v),
        }
    }
}

impl Display for GroupName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.as_ref().map(|g| g.as_str()).unwrap_or("none")
        )
    }
}

pub trait EmitStateChange {
    fn emit(&mut self, sc: StateChange);
}

pub trait EmitFixtureStateChange {
    fn emit(&mut self, sc: FixtureStateChange);

    fn emit_comet(&mut self, sc: CometStateChange) {
        self.emit(FixtureStateChange::Comet(sc));
    }

    fn emit_lumasphere(&mut self, sc: LumasphereStateChange) {
        self.emit(FixtureStateChange::Lumasphere(sc));
    }

    fn emit_venus(&mut self, sc: VenusStateChange) {
        self.emit(FixtureStateChange::Venus(sc));
    }

    fn emit_h2o(&mut self, sc: H2OStateChange) {
        self.emit(FixtureStateChange::H2O(sc));
    }

    fn emit_aquarius(&mut self, sc: AquariusStateChange) {
        self.emit(FixtureStateChange::Aquarius(sc));
    }

    fn emit_radiance(&mut self, sc: RadianceStateChange) {
        self.emit(FixtureStateChange::Radiance(sc));
    }

    fn emit_swarmolon(&mut self, sc: SwarmolonStateChange) {
        self.emit(FixtureStateChange::Swarmolon(sc));
    }

    fn emit_rotosphere_q3(&mut self, sc: RotosphereQ3StateChange) {
        self.emit(FixtureStateChange::RotosphereQ3(sc));
    }

    fn emit_freedom_fries(&mut self, sc: FreedomFriesStateChange) {
        self.emit(FixtureStateChange::FreedomFries(sc));
    }

    fn emit_faderboard(&mut self, sc: FaderboardStateChange) {
        self.emit(FixtureStateChange::Faderboard(sc));
    }

    fn emit_rush_wizard(&mut self, sc: RushWizardStateChange) {
        self.emit(FixtureStateChange::RushWizard(sc));
    }

    fn emit_wizard_extreme(&mut self, sc: WizardExtremeStateChange) {
        self.emit(FixtureStateChange::WizardExtreme(sc));
    }

    fn emit_color(&mut self, sc: ColorStateChange) {
        self.emit(FixtureStateChange::Color(sc));
    }

    fn emit_dimmer(&mut self, sc: DimmerStateChange) {
        self.emit(FixtureStateChange::Dimmer(sc));
    }
}

#[derive(Clone, Debug)]
pub struct StateChange {
    pub group: GroupName,
    pub sc: FixtureStateChange,
}

#[derive(Clone, Debug)]
pub enum FixtureStateChange {
    Comet(CometStateChange),
    Lumasphere(LumasphereStateChange),
    Venus(VenusStateChange),
    H2O(H2OStateChange),
    Aquarius(AquariusStateChange),
    Radiance(RadianceStateChange),
    Swarmolon(SwarmolonStateChange),
    RotosphereQ3(RotosphereQ3StateChange),
    FreedomFries(FreedomFriesStateChange),
    Faderboard(FaderboardStateChange),
    RushWizard(RushWizardStateChange),
    WizardExtreme(WizardExtremeStateChange),
    Color(ColorStateChange),
    Dimmer(DimmerControlMessage),
    Master(MasterStateChange),
}

#[derive(Clone, Debug)]
pub struct ControlMessage {
    pub group: GroupName,
    pub msg: FixtureControlMessage,
}

#[derive(Clone, Debug)]
pub enum FixtureControlMessage {
    Comet(CometControlMessage),
    Lumasphere(LumasphereControlMessage),
    Venus(VenusControlMessage),
    H2O(H2OControlMessage),
    Aquarius(AquariusControlMessage),
    Radiance(RadianceControlMessage),
    Swarmolon(SwarmolonControlMessage),
    RotosphereQ3(RotosphereQ3ControlMessage),
    FreedomFries(FreedomFriesControlMessage),
    Faderboard(FaderboardControlMessage),
    RushWizard(RushWizardControlMessage),
    WizardExtreme(WizardExtremeControlMessage),
    Color(ColorControlMessage),
    Dimmer(DimmerControlMessage),
    Master(MasterControlMessage),
}

pub const N_ANIM: usize = 4;

#[derive(Debug)]
pub struct FixtureGroup {
    /// The name of this type of fixture.
    fixture_type: String,
    /// The group index.
    name: GroupName,
    /// The starting index into the DMX buffer for the fixtures in this group.
    dmx_indexes: Vec<usize>,
    /// The number of DMX channels used by this fixture.
    channel_count: usize,
    /// The inner implementation of the fixture.
    fixture: Box<dyn Fixture>,
    /// Optional collection of animations.
    animations: Option<[TargetedAnimation; N_ANIM]>,
}

impl FixtureGroup {
    pub fn name(&self) -> &str {
        &self.fixture_type
    }

    /// Emit the current state of all controls.
    pub fn emit_state(&self, emitter: &mut dyn EmitStateChange) {
        let mut emitter = StateChangeWithGroupEmitter {
            emitter,
            group: self.name.clone(),
        };
        self.fixture.emit_state(&mut emitter);
    }

    /// Potentially process the provided control message.
    /// If this fixture will not process it, return it back to the caller.
    pub fn control(
        &mut self,
        msg: ControlMessage,
        emitter: &mut dyn EmitStateChange,
    ) -> Option<ControlMessage> {
        if self.name != msg.group {
            return Some(msg);
        }
        let mut emitter = StateChangeWithGroupEmitter {
            emitter,
            group: self.name.clone(),
        };
        self.fixture
            .control(msg.msg, &mut emitter)
            .map(|m| ControlMessage {
                group: msg.group,
                msg: m,
            })
    }

    pub fn update(&mut self, delta_t: Duration) {
        self.fixture.update(delta_t);
    }

    /// Render into the provided DMX universe.
    /// The master controls are provided to potentially alter the render.
    pub fn render(&self, master_controls: &MasterControls, dmx_univ: &mut [u8]) {
        let phase_offset_per_fixture = Phase::new(1.0 / self.dmx_indexes.len() as f64);
        let mut animation_vals = [(0.0, AnimationTarget::None); N_ANIM];
        for (i, dmx_index) in self.dmx_indexes.iter().enumerate() {
            let phase_offset = phase_offset_per_fixture * i as f64;
            // FIXME: implement unipolar variant of animations
            if let Some(animations) = self.animations.as_ref() {
                for (i, ta) in animations.iter().enumerate() {
                    animation_vals[i] = (
                        ta.animation.get_value(
                            phase_offset,
                            &master_controls.clock_state,
                            UnipolarFloat::ZERO,
                        ),
                        ta.target,
                    );
                }
            }
            let dmx_buf = &mut dmx_univ[*dmx_index..*dmx_index + self.channel_count];
            self.fixture
                .render_with_animations(master_controls, &animation_vals, dmx_buf);
            debug!("{} ({}): {:?}", self.fixture_type, self.name, dmx_buf);
        }
    }
}

/// Wrap a state change emitter,
struct StateChangeWithGroupEmitter<'a> {
    emitter: &'a mut dyn EmitStateChange,
    group: GroupName,
}

impl<'a> EmitFixtureStateChange for StateChangeWithGroupEmitter<'a> {
    fn emit(&mut self, sc: FixtureStateChange) {
        self.emitter.emit(StateChange {
            group: self.group.clone(),
            sc,
        });
    }
}

impl MapControls for FixtureGroup {
    fn map_controls(&self, map: &mut crate::osc::ControlMap<FixtureControlMessage>) {
        self.fixture.map_controls(map);
    }
}

type UsedAddrs = HashMap<usize, FixtureConfig>;

pub struct Patch {
    fixtures: Vec<FixtureGroup>,
    used_addrs: UsedAddrs,
}

impl Patch {
    pub fn new() -> Self {
        Self {
            fixtures: Vec::new(),
            used_addrs: HashMap::new(),
        }
    }

    pub fn patch(&mut self, cfg: FixtureConfig) -> Result<(), Box<dyn Error>> {
        let candidate = match cfg.name.as_str() {
            "comet" => Comet::patch(&cfg),
            "lumasphere" => Lumasphere::patch(&cfg),
            "venus" => Venus::patch(&cfg),
            "h2o" => H2O::patch(&cfg),
            "aquarius" => Aquarius::patch(&cfg),
            "radiance" => Radiance::patch(&cfg),
            "swarmolon" => Swarmolon::patch(&cfg),
            "rotosphere_q3" => RotosphereQ3::patch(&cfg),
            "freedom_fries" => FreedomFries::patch(&cfg),
            "faderboard" => Faderboard::patch(&cfg),
            "rush_wizard" => RushWizard::patch(&cfg),
            "wizard_extreme" => WizardExtreme::patch(&cfg),
            "color" => Color::patch(&cfg),
            "dimmer" => Dimmer::patch(&cfg),
            unknown => {
                bail!("Unknown fixture type \"{}\".", unknown);
            }
        }?;
        self.used_addrs = self.check_collision(&candidate, &cfg)?;
        info!(
            "Controlling {} at {} (group: {}).",
            cfg.name, cfg.addr, cfg.group
        );
        // Either identify an existing appropriate group or create a new one.
        for group in self.fixtures.iter_mut() {
            if group.fixture_type == cfg.name && group.name == cfg.group {
                group.dmx_indexes.push(cfg.addr.dmx_index());
                return Ok(());
            }
        }
        // No existing group; create a new one.
        self.fixtures.push(FixtureGroup {
            fixture_type: cfg.name,
            name: cfg.group,
            dmx_indexes: vec![cfg.addr.dmx_index()],
            channel_count: candidate.channel_count,
            fixture: candidate.fixture,
            animations: None,
        });

        Ok(())
    }

    /// Check that the patch candidate doesn't conflict with another patched fixture.
    /// Return an updated collection of used addresses if it does not conflict.
    fn check_collision(
        &self,
        candidate: &PatchCandidate,
        cfg: &FixtureConfig,
    ) -> Result<UsedAddrs, Box<dyn Error>> {
        let mut used_addrs = self.used_addrs.clone();
        let dmx_index = cfg.addr.dmx_index();
        for addr in dmx_index..dmx_index + candidate.channel_count {
            match used_addrs.get(&addr) {
                Some(existing_fixture) => {
                    bail!(
                        "{} at {} overlaps at DMX address {} with {} at {}.",
                        cfg.name,
                        cfg.addr,
                        addr + 1,
                        existing_fixture.name,
                        existing_fixture.addr,
                    );
                }
                None => {
                    used_addrs.insert(addr, cfg.clone());
                }
            }
        }
        Ok(used_addrs)
    }

    pub fn iter(&self) -> impl Iterator<Item = &FixtureGroup> {
        self.fixtures.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut FixtureGroup> {
        self.fixtures.iter_mut()
    }
}

pub struct PatchCandidate {
    fixture: Box<dyn Fixture>,
    channel_count: usize,
}

/// Fixture constructor trait to handle patching fixtures.
pub trait PatchFixture: Fixture + Default {
    /// Create a new instance of the fixture from the provided options.
    /// Non-customizable fixtures will fall back to using default.
    /// This can be overridden for fixtures that are customizable.
    fn new(_options: &HashMap<String, String>) -> Result<Self, Box<dyn Error>> {
        Ok(Self::default())
    }

    /// The number of contiguous DMX channels used by the fixture.
    fn channel_count(&self) -> usize;

    /// Produce a wrapped fixture at the provided DMX address.
    fn patch(cfg: &FixtureConfig) -> Result<PatchCandidate, Box<dyn Error>>
    where
        Self: Sized + 'static,
    {
        let fixture = Self::new(&cfg.options)?;
        Ok(PatchCandidate {
            channel_count: fixture.channel_count(),
            fixture: Box::new(fixture),
        })
    }
}

pub trait Fixture: MapControls + Debug {
    /// Emit the current state of all controls.
    fn emit_state(&self, emitter: &mut dyn EmitFixtureStateChange);

    /// Potentially process the provided control message.
    /// If this fixture will not process it, return it back to the caller.
    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitFixtureStateChange,
    ) -> Option<FixtureControlMessage>;

    fn update(&mut self, _: Duration) {}

    /// Render into the provided DMX buffer, including animations.
    /// This default implementation ignores animations.
    fn render_with_animations(
        &self,
        master_controls: &MasterControls,
        _animations: &TargetedAnimations,
        dmx_buffer: &mut [u8],
    ) {
        self.render(master_controls, dmx_buffer);
    }

    /// Render into the provided DMX buffer.
    /// The buffer will be pre-sized to the fixture's channel count and offset
    /// to the fixture's start address.
    /// The master controls are provided to potentially alter the render process.
    fn render(&self, _master_controls: &MasterControls, _dmx_buffer: &mut [u8]) {
        // FIXME: no-op default to allow implementing either render or
        // render_with_animations...
    }
}
