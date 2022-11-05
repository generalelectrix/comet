use crate::dmx::DmxAddr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub receive_port: u16,
    pub send_host: String,
    pub send_port: u16,
    pub debug: bool,
    pub fixtures: Vec<FixtureConfig>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
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
    pub group: Option<String>,
    /// Additional key-value string options for configuring specific fixture types.
    #[serde(default)]
    pub options: HashMap<String, String>,
}
