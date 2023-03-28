use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use log::{debug, info};
use simple_error::bail;

use self::animation_target::{TargetedAnimations};
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

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
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

pub struct FixtureGroup {
    /// The name of this group.  If None, it is the "default" group.
    name: GroupName,
    // /// Animators defined for this group, if any.
    // animators:
}

#[derive(Debug)]
pub struct FixtureWrapper {
    /// The name of this type of fixture.
    name: String,
    /// The group index of this fixture.
    group: GroupName,
    /// The starting index into the DMX buffer for this fixture.
    dmx_index: usize,
    /// The number of DMX channels used by this fixture.
    channel_count: usize,
    /// The inner implementation of the fixture.
    fixture: Box<dyn Fixture>,
}

impl FixtureWrapper {
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Emit the current state of all controls.
    pub fn emit_state(&self, emitter: &mut dyn EmitStateChange) {
        let mut emitter = StateChangeWithGroupEmitter {
            emitter,
            group: self.group.clone(),
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
        if self.group != msg.group {
            return Some(msg);
        }
        let mut emitter = StateChangeWithGroupEmitter {
            emitter,
            group: self.group.clone(),
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
    pub fn render(
        &self,
        master_controls: &MasterControls,
        animations: &TargetedAnimations,
        dmx_univ: &mut [u8],
    ) {
        let dmx_buf = &mut dmx_univ[self.dmx_index..self.dmx_index + self.channel_count];
        self.fixture
            .render_with_animations(master_controls, animations, dmx_buf);
        debug!("{} ({:?}): {:?}", self.name, self.group, dmx_buf);
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

impl MapControls for FixtureWrapper {
    fn map_controls(&self, map: &mut crate::osc::ControlMap<FixtureControlMessage>) {
        self.fixture.map_controls(map);
    }
}

pub struct Patch {
    fixtures: Vec<FixtureWrapper>,
    used_addrs: HashMap<usize, FixtureConfig>,
    used_groups: HashMap<String, HashSet<GroupName>>,
}

impl Patch {
    pub fn new() -> Self {
        Self {
            fixtures: Vec::new(),
            used_addrs: HashMap::new(),
            used_groups: HashMap::new(),
        }
    }

    pub fn patch(&mut self, cfg: &FixtureConfig) -> Result<(), Box<dyn Error>> {
        let fixture = match cfg.name.as_str() {
            "comet" => Comet::patch(cfg),
            "lumasphere" => Lumasphere::patch(cfg),
            "venus" => Venus::patch(cfg),
            "h2o" => H2O::patch(cfg),
            "aquarius" => Aquarius::patch(cfg),
            "radiance" => Radiance::patch(cfg),
            "swarmolon" => Swarmolon::patch(cfg),
            "rotosphere_q3" => RotosphereQ3::patch(cfg),
            "freedom_fries" => FreedomFries::patch(cfg),
            "faderboard" => Faderboard::patch(cfg),
            "rush_wizard" => RushWizard::patch(cfg),
            "wizard_extreme" => WizardExtreme::patch(cfg),
            "color" => Color::patch(cfg),
            "dimmer" => Dimmer::patch(cfg),
            unknown => {
                bail!("Unknown fixture type \"{}\".", unknown);
            }
        }?;
        self.check_collision(&fixture, cfg)?;
        self.fixtures.push(fixture);
        info!(
            "Controlling {} at {} (group: {:?}).",
            cfg.name, cfg.addr, cfg.group
        );
        Ok(())
    }

    fn check_collision(
        &mut self,
        fixture: &FixtureWrapper,
        cfg: &FixtureConfig,
    ) -> Result<(), Box<dyn Error>> {
        for addr in fixture.dmx_index..fixture.dmx_index + fixture.channel_count {
            match self.used_addrs.get(&addr) {
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
                    self.used_addrs.insert(addr, cfg.clone());
                }
            }
        }
        match self.used_groups.get_mut(&fixture.name) {
            Some(existing_groups) if existing_groups.contains(&fixture.group) => {
                bail!(
                    "duplicate group declaration for fixture type {}",
                    fixture.name
                );
            }
            Some(existing_groups) => {
                existing_groups.insert(fixture.group.clone());
            }
            None => {
                let mut set = HashSet::new();
                set.insert(fixture.group.clone());
                self.used_groups.insert(cfg.name.clone(), set);
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &FixtureWrapper> {
        self.fixtures.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut FixtureWrapper> {
        self.fixtures.iter_mut()
    }
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
    fn patch(cfg: &FixtureConfig) -> Result<FixtureWrapper, Box<dyn Error>>
    where
        Self: Sized + 'static,
    {
        let fixture = Self::new(&cfg.options)?;
        Ok(FixtureWrapper {
            name: cfg.name.clone(),
            group: (&cfg.group).into(),
            dmx_index: cfg.addr - 1,
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
    fn render(&self, master_controls: &MasterControls, dmx_buffer: &mut [u8]);
}
