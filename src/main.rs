use local_ip_address::local_ip;
use log::info;
use log::LevelFilter;
use rust_dmx::select_port;
use simplelog::{Config as LogConfig, SimpleLogger};
use std::env;
use std::error::Error;

use crate::config::Config;
use crate::show::Show;

mod config;
mod dmx;
mod fixture;
mod master;
mod osc;
mod show;
mod util;

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

    match local_ip() {
        Ok(ip) => info!("Listening for OSC at {}:{}.", ip, cfg.receive_port),
        Err(e) => info!("Unable to fetch local IP address: {}.", e),
    }

    let mut show = Show::new(cfg)?;

    let dmx_port = select_port()?;

    show.run(dmx_port);

    Ok(())
}
