use crate::dmx::DmxAddr;
use crate::fixture::GroupName;
use crate::osc::OscSenderConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub receive_port: u16,
    pub controllers: Vec<OscSenderConfig>,
    pub debug: bool,
    pub fixtures: Vec<FixtureConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimationGroup {
    pub fixture_type: String,
    pub group: GroupName,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let config_file = File::open(path)?;
        let cfg: Config = serde_yaml::from_reader(config_file)?;
        Ok(cfg)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FixtureConfig {
    pub name: String,
    pub addr: DmxAddr,
    /// For fixtures that we may want to separately control multiple instances,
    /// provide a group index.  Most fixtures do not use this.
    #[serde(default)]
    pub group: GroupName,
    /// Additional key-value string options for configuring specific fixture types.
    #[serde(default)]
    pub options: Options,
    /// If true, use animations.
    #[serde(default)]
    pub animations: bool,
    /// If present, assign to this selector index.
    #[serde(default)]
    pub selector: Option<usize>,
}

pub type Options = HashMap<String, String>;
