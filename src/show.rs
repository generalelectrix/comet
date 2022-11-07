use std::{
    collections::HashSet,
    error::Error,
    time::{Duration, Instant},
};

use crate::{
    config::Config,
    fixture::{FixtureControlMessage, Patch},
    master::MasterControls,
    osc::OscController,
};

use log::{error, warn};
use rust_dmx::DmxPort;

pub struct Show {
    osc_controller: OscController,
    patch: Patch,
    master_controls: MasterControls,
}

const CONTROL_TIMEOUT: Duration = Duration::from_millis(1);
const UPDATE_INTERVAL: Duration = Duration::from_millis(10);

impl Show {
    pub fn new(cfg: &Config) -> Result<Self, Box<dyn Error>> {
        let mut patch = Patch::new();

        let mut osc_controller =
            OscController::new(cfg.receive_port, &cfg.send_host, cfg.send_port)?;

        for fixture in cfg.fixtures.iter() {
            patch.patch(fixture)?;
        }

        // Only patch a fixture type's controls once.
        let mut patched_controls = HashSet::new();

        for fixture in patch.iter() {
            if !patched_controls.contains(fixture.name()) {
                osc_controller.map_controls(fixture);
                patched_controls.insert(fixture.name().to_string());
            }

            fixture.emit_state(&mut osc_controller);
        }

        let master_controls = MasterControls::default();
        osc_controller.map_controls(&master_controls);
        master_controls.emit_state(&mut osc_controller);

        Ok(Self {
            patch,
            osc_controller,
            master_controls,
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
        let msg = match self.osc_controller.recv(timeout)? {
            Some(m) => m,
            None => {
                return Ok(());
            }
        };

        if let FixtureControlMessage::Master(mc) = msg.msg {
            self.master_controls.control(mc, &mut self.osc_controller);
            return Ok(());
        }

        // "Option dance" to pass ownership into/back out of handlers.
        let mut msg = Some(msg);

        for fixture in self.patch.iter_mut() {
            match msg.take() {
                Some(m) => {
                    msg = fixture.control(m, &mut self.osc_controller);
                }
                None => {
                    break;
                }
            }
        }
        if let Some(m) = msg {
            warn!("Control message was not handled by any fixture: {:?}", m);
        }
        Ok(())
    }

    fn update(&mut self, delta_t: Duration) {
        self.master_controls.update(delta_t);
        for fixture in self.patch.iter_mut() {
            fixture.update(delta_t);
        }
    }

    fn render(&self, dmx_buffer: &mut [u8]) {
        // NOTE: we don't bother to empty the buffer because we will always
        // overwrite all previously-rendered state.
        for fixture in self.patch.iter() {
            fixture.render(&self.master_controls, dmx_buffer);
        }
    }
}
