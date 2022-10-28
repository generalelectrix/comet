use crossbeam_channel::unbounded;
use dmx::DmxAddr;
use local_ip_address::local_ip;
use log::info;
use log::LevelFilter;
use rust_dmx::select_port;
use serde::{Deserialize, Serialize};
use simplelog::{Config as LogConfig, SimpleLogger};
use std::cmp;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::time::Duration;

use crate::show::Show;

mod comet;
mod dmx;
mod event;
mod fixture;
mod lumasphere;
mod osc;
mod show;
mod util;

fn main() -> Result<(), Box<dyn Error>> {
    let config_path = env::args()
        .nth(1)
        .expect("Provide config path as first arg.");
    let mut cfg = Config::load(&config_path)?;
    let log_level = if cfg.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Error
    };
    SimpleLogger::init(log_level, LogConfig::default())?;

    let mut dmx_port = select_port()?;

    let ip = local_ip()?;
    info!("Listening at {}", ip);

    let show = Show::new(&cfg)?;

    println!("{:?}", cfg);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    receive_port: u16,
    send_host: String,
    send_port: u16,
    dmx_addr: DmxAddr,
    debug: bool,
    update_interval: i64,
    fixture: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let config_file = File::open(path)?;
        let cfg: Config = serde_yaml::from_reader(config_file)?;
        Ok(cfg)
    }
}
