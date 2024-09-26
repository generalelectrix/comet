use anyhow::bail;
use clock_service::prompt_start_clock_service;
use local_ip_address::local_ip;
use log::info;
use log::LevelFilter;
use osc::prompt_osc_config;
use rust_dmx::select_port;
use simplelog::{Config as LogConfig, SimpleLogger};
use std::env;
use zmq::Context;

use crate::config::Config;
use crate::show::Show;

mod animation;
mod clock_service;
mod config;
mod dmx;
mod fixture;
mod master;
mod osc;
mod show;
mod util;

fn main() -> anyhow::Result<()> {
    let config_path = env::args()
        .nth(1)
        .expect("Provide config path as first arg.");
    let mut cfg = Config::load(&config_path)?;
    println!("{cfg:?}");
    let log_level = if cfg.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    SimpleLogger::init(log_level, LogConfig::default())?;
    let clock_service = prompt_start_clock_service(Context::new())?;

    match local_ip() {
        Ok(ip) => info!("Listening for OSC at {}:{}.", ip, cfg.receive_port),
        Err(e) => info!("Unable to fetch local IP address: {}.", e),
    }

    if let Some(clients) = prompt_osc_config(cfg.receive_port)? {
        cfg.controllers = clients;
    }
    if cfg.controllers.is_empty() {
        bail!("No OSC clients were registered or manually configured.");
    }

    let mut show = Show::new(cfg, clock_service)?;

    let universe_count = show.universe_count();
    println!("This show requires {universe_count} universes.");

    let mut dmx_ports = Vec::new();

    for i in 0..universe_count {
        println!("Assign port to universe {i}:");
        dmx_ports.push(select_port()?);
    }

    show.run(&mut dmx_ports);

    Ok(())
}
