use anyhow::bail;
use clock_service::prompt_start_clock_service;
use local_ip_address::local_ip;
use log::info;
use log::LevelFilter;
use midi::Device;
use number::UnipolarFloat;
use osc::prompt_osc_config;
use osc::GroupControlMap;
use rust_dmx::select_port;
use show::Clocks;
use simplelog::{Config as LogConfig, SimpleLogger};
use std::env;
use tunnels::audio::prompt_audio;
use tunnels::audio::AudioInput;
use tunnels::clock_bank::ClockBank;
use tunnels::midi::list_ports;
use tunnels::midi::prompt_midi;
use zmq::Context;

use crate::config::Config;
use crate::show::Show;

mod animation;
mod channel;
mod clock_service;
mod config;
mod control;
mod dmx;
mod fixture;
mod master;
mod midi;
mod osc;
mod show;
mod util;
mod wled;

fn main() -> anyhow::Result<()> {
    let config_path = env::args()
        .nth(1)
        .expect("Provide config path as first arg.");
    let mut cfg = Config::load(&config_path)?;
    let log_level = if cfg.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    SimpleLogger::init(log_level, LogConfig::default())?;
    let clocks = if let Some(clock_service) = prompt_start_clock_service(Context::new())? {
        Clocks::Service(clock_service)
    } else {
        let audio_input = AudioInput::new(prompt_audio()?)?;
        let clocks = ClockBank::default();
        let mut audio_controls = GroupControlMap::default();
        crate::osc::audio::map_controls(&mut audio_controls);
        Clocks::Internal {
            clocks,
            audio_input,
            audio_controls,
        }
    };

    match local_ip() {
        Ok(ip) => info!("Listening for OSC at {}:{}.", ip, cfg.receive_port),
        Err(e) => info!("Unable to fetch local IP address: {}.", e),
    }

    if let Some(clients) = prompt_osc_config(cfg.receive_port)? {
        cfg.controllers = clients;
    }
    let (midi_inputs, midi_outputs) = list_ports()?;
    cfg.midi_devices = prompt_midi(&midi_inputs, &midi_outputs, Device::all())?;
    if cfg.controllers.is_empty() && cfg.midi_devices.is_empty() {
        bail!("No OSC or midi clients were registered or manually configured.");
    }

    let mut show = Show::new(cfg, clocks)?;

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
