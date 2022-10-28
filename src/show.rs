use std::{
    error::Error,
    time::{Duration, Instant},
};

use crate::{
    comet::Comet, fixture::ControlMessage, lumasphere::Lumasphere, osc::OscController, Config,
};
use log::error;
use rust_dmx::DmxPort;
use simple_error::bail;

pub struct Show {
    osc_controller: OscController,
    dmx_buffer: Vec<u8>,
    comet: Option<Comet>,
    lumasphere: Option<Lumasphere>,
}

const CONTROL_TIMEOUT: Duration = Duration::from_millis(1);
const UPDATE_INTERVAL: Duration = Duration::from_millis(10);

impl Show {
    pub fn new(cfg: &Config) -> Result<Self, Box<dyn Error>> {
        let mut comet = None;
        let mut lumasphere = None;

        let mut osc_controller =
            OscController::new(cfg.receive_port, &cfg.send_host, cfg.send_port)?;

        match cfg.fixture.as_str() {
            "comet" => {
                comet = Some(Comet::new(cfg.dmx_addr));
                osc_controller.map_comet_controls();
                println!("Controlling the Comet.");
            }
            "lumasphere" => {
                lumasphere = Some(Lumasphere::new(cfg.dmx_addr));
                osc_controller.map_lumasphere_controls();
                println!("Controlling the Lumasphere.");
            }
            unknown => {
                bail!("Unknown fixture type \"{}\".", unknown);
            }
        };

        Ok(Self {
            comet,
            lumasphere,
            osc_controller,
            dmx_buffer: vec![0u8; 512],
        })
    }

    /// Run the show forever in the current thread.
    pub fn run(&mut self, mut dmx_port: Box<dyn DmxPort>) {
        let mut update_number = 0;

        let mut last_update = Instant::now();
        let mut last_rendered_frame = 0;
        let mut dmx_buffer = vec![0u8; 512];
        loop {
            // Process a control event if one is pending.
            if let Err(err) = self.control(CONTROL_TIMEOUT) {
                error!("A control error occurred: {}.", err);
            }

            // Compute updates until we're current.
            let mut now = Instant::now();
            let mut time_since_last_update = now - last_update;
            while time_since_last_update > UPDATE_INTERVAL {
                // Update the state of the show.
                self.update(UPDATE_INTERVAL);

                last_update += UPDATE_INTERVAL;
                now = Instant::now();
                time_since_last_update = now - last_update;
                update_number += 1;
            }

            // Render the state of the show.
            self.render(&mut dmx_buffer);
            if let Err(e) = dmx_port.write(&dmx_buffer) {
                error!("DMX write error: {}.", e);
            }
        }
    }

    fn control(&mut self, timeout: Duration) -> Result<(), Box<dyn Error>> {
        let msg = match self.osc_controller.recv(timeout)? {
            Some(m) => m,
            None => {
                return Ok(());
            }
        };
        match msg {
            ControlMessage::Comet(c) => {
                self.comet
                    .as_mut()
                    .map(|comet| comet.control(c, &mut self.osc_controller));
            }
            ControlMessage::Lumasphere(c) => {
                self.lumasphere
                    .as_mut()
                    .map(|lumasphere| lumasphere.control(c, &mut self.osc_controller));
            }
        }
        Ok(())
    }

    fn update(&mut self, delta_t: Duration) {
        self.comet.as_mut().map(|c| c.update(delta_t));
        self.lumasphere.as_mut().map(|l| l.update(delta_t));
    }

    fn render(&mut self, dmx_buffer: &mut [u8]) {
        // NOTE: we don't bother to empty the buffer because we will always
        // overwrite all previously-rendered state.
        self.comet.as_ref().map(|c| c.render(dmx_buffer));
        self.lumasphere.as_ref().map(|l| l.render(dmx_buffer));
    }
}
