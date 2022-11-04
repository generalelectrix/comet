use std::{
    error::Error,
    time::{Duration, Instant},
};

use crate::{
    aquarius::Aquarius, comet::Comet, freedom_fries::FreedomFries, h2o::H2O,
    lumasphere::Lumasphere, osc::OscController, rotosphere_q3::RotosphereQ3, swarmolon::Swarmolon,
    venus::Venus, Config,
};
use crate::{fixture::Fixture, radiance::Radiance};
use log::error;
use rust_dmx::DmxPort;
use simple_error::bail;

pub struct Show {
    osc_controller: OscController,
    fixtures: Vec<Box<dyn Fixture>>,
}

const CONTROL_TIMEOUT: Duration = Duration::from_millis(1);
const UPDATE_INTERVAL: Duration = Duration::from_millis(10);

impl Show {
    pub fn new(mut cfg: Config) -> Result<Self, Box<dyn Error>> {
        let mut fixtures: Vec<Box<dyn Fixture>> = Vec::new();

        let mut osc_controller =
            OscController::new(cfg.receive_port, &cfg.send_host, cfg.send_port)?;

        for (fixture, addrs) in cfg.fixtures.drain() {
            match fixture.as_str() {
                "comet" => {
                    let fixture = Comet::new(addrs[0]);
                    osc_controller.map_comet_controls();
                    fixtures.push(Box::new(fixture));
                    println!("Controlling the Comet.");
                }
                "lumasphere" => {
                    let fixture = Lumasphere::new(addrs[0]);
                    osc_controller.map_lumasphere_controls();
                    fixtures.push(Box::new(fixture));
                    println!("Controlling the Lumasphere.");
                }
                "venus" => {
                    let fixture = Venus::new(addrs[0]);
                    osc_controller.map_venus_controls();
                    fixtures.push(Box::new(fixture));
                    println!("Controlling the Venus.");
                }
                "h2o" => {
                    let fixture = H2O::new(addrs);
                    osc_controller.map_h2o_controls();
                    fixtures.push(Box::new(fixture));
                    println!("Controlling H2Os.");
                }
                "aquarius" => {
                    let fixture = Aquarius::new(addrs);
                    osc_controller.map_aquarius_controls();
                    fixtures.push(Box::new(fixture));
                    println!("Controlling Aquarii.");
                }
                "radiance" => {
                    let fixture = Radiance::new(addrs[0]);
                    osc_controller.map_radiance_controls();
                    fixtures.push(Box::new(fixture));
                    println!("Controlling a Radiance.");
                }
                "swarmolon" => {
                    let fixture = Swarmolon::new(addrs, true);
                    osc_controller.map_swarmolon_controls();
                    fixtures.push(Box::new(fixture));
                    println!("Controlling Swarmolons.");
                }
                "rotosphere_q3" => {
                    let fixture = RotosphereQ3::new(addrs[0]);
                    osc_controller.map_rotosphere_q3_controls();
                    fixtures.push(Box::new(fixture));
                    println!("Controlling Rotosphere Q3.");
                }
                "freedom_fries" => {
                    let fixture = FreedomFries::new(addrs[0]);
                    osc_controller.map_freedom_fries_controls();
                    fixtures.push(Box::new(fixture));
                    println!("Controlling Freedom Fries.");
                }
                unknown => {
                    bail!("Unknown fixture type \"{}\".", unknown);
                }
            }
        }

        for fixture in fixtures.iter() {
            fixture.emit_state(&mut osc_controller);
        }

        Ok(Self {
            fixtures,
            osc_controller,
        })
    }

    /// Run the show forever in the current thread.
    pub fn run(&mut self, mut dmx_port: Box<dyn DmxPort>) {
        let mut last_update = Instant::now();
        let mut dmx_buffer = vec![0u8; 512];
        loop {
            // Process a control event if one is pending.
            if let Err(err) = self.control(CONTROL_TIMEOUT) {
                error!("A control error occurred: {}.", err);
            }

            // Compute updates until we're current.
            let mut now = Instant::now();
            let mut time_since_last_update = now - last_update;
            let mut should_render = false;
            while time_since_last_update > UPDATE_INTERVAL {
                // Update the state of the show.
                self.update(UPDATE_INTERVAL);
                should_render = true;

                last_update += UPDATE_INTERVAL;
                now = Instant::now();
                time_since_last_update = now - last_update;
            }

            // Render the state of the show.
            if should_render {
                self.render(&mut dmx_buffer);
                if let Err(e) = dmx_port.write(&dmx_buffer) {
                    error!("DMX write error: {}.", e);
                }
            }
        }
    }

    fn control(&mut self, timeout: Duration) -> Result<(), Box<dyn Error>> {
        let mut msg = Some(match self.osc_controller.recv(timeout)? {
            Some(m) => m,
            None => {
                return Ok(());
            }
        });
        for fixture in self.fixtures.iter_mut() {
            match msg.take() {
                Some(m) => {
                    msg = fixture.control(m, &mut self.osc_controller);
                }
                None => {
                    break;
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_t: Duration) {
        for fixture in self.fixtures.iter_mut() {
            fixture.update(delta_t);
        }
    }

    fn render(&mut self, dmx_buffer: &mut [u8]) {
        // NOTE: we don't bother to empty the buffer because we will always
        // overwrite all previously-rendered state.
        for fixture in self.fixtures.iter() {
            fixture.render(dmx_buffer);
        }
    }
}
