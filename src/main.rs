use dmx::DmxAddr;
use local_ip_address::local_ip;
use log::info;
use log::LevelFilter;
use rust_dmx::select_port;
use serde::{Deserialize, Serialize};
use simplelog::{Config as LogConfig, SimpleLogger};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;

use crate::show::Show;

mod aquarius;
mod comet;
mod dmx;
mod fixture;
mod generic;
mod h2o;
mod lumasphere;
mod osc;
mod radiance;
mod rotosphere_q3;
mod show;
mod swarmolon;
mod util;
mod venus;

fn main() -> Result<(), Box<dyn Error>> {
    let config_path = env::args()
        .nth(1)
        .expect("Provide config path as first arg.");
    let cfg = Config::load(&config_path)?;
    let log_level = if cfg.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    SimpleLogger::init(log_level, LogConfig::default())?;

    let ip = local_ip()?;
    info!("Listening for OSC at {}:{}", ip, cfg.receive_port);

    let dmx_port = select_port()?;

    let mut show = Show::new(cfg)?;

    show.run(dmx_port);

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    receive_port: u16,
    send_host: String,
    send_port: u16,
    debug: bool,
    fixtures: HashMap<String, Vec<DmxAddr>>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let config_file = File::open(path)?;
        let cfg: Config = serde_yaml::from_reader(config_file)?;
        Ok(cfg)
    }
}
