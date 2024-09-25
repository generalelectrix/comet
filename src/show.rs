use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use crate::{
    animation::AnimationUIState,
    clock_service::ClockService,
    config::Config,
    dmx::DmxBuffer,
    fixture::{ControlMessagePayload, FixtureGroup, Patch},
    master::MasterControls,
    osc::OscController,
};

use anyhow::{bail, Result};
use log::error;
use number::UnipolarFloat;
use rust_dmx::DmxPort;

pub struct Show {
    osc_controller: OscController,
    patch: Patch,
    master_controls: MasterControls,
    animation_ui_state: AnimationUIState,
    clock_service: Option<ClockService>,
}

const CONTROL_TIMEOUT: Duration = Duration::from_millis(1);
const UPDATE_INTERVAL: Duration = Duration::from_millis(20);

impl Show {
    pub fn new(cfg: Config, clock_service: Option<ClockService>) -> Result<Self> {
        let mut patch = Patch::default();

        let mut osc_controller = OscController::new(cfg.receive_port, &cfg.controllers)?;

        for fixture in cfg.fixtures.into_iter() {
            patch.patch(fixture)?;
        }

        // Only patch a fixture type's controls once.
        let mut patched_controls = HashSet::new();

        for group in patch.iter() {
            if !patched_controls.contains(group.fixture_type()) {
                osc_controller.map_controls(group);
                patched_controls.insert(group.fixture_type());
            }

            group.emit_state(&osc_controller);
        }

        let master_controls = MasterControls::default();
        osc_controller.map_controls(&master_controls);
        master_controls.emit_state(&osc_controller);

        // Configure animation controls.
        let animation_ui_state = if patch.iter().any(FixtureGroup::is_animated) {
            let state = AnimationUIState::new(Some(patch.validate_selector(0)?));
            osc_controller.map_controls(&state);
            state.emit_state(&mut patch, &osc_controller)?;
            state
        } else {
            AnimationUIState::new(None)
        };

        Ok(Self {
            patch,
            osc_controller,
            master_controls,
            animation_ui_state,
            clock_service,
        })
    }

    /// Return the number of universes patched in the show.
    pub fn universe_count(&self) -> usize {
        self.patch.universe_count()
    }

    /// Run the show forever in the current thread.
    pub fn run(&mut self, dmx_ports: &mut [Box<dyn DmxPort>]) {
        let mut last_update = Instant::now();
        let mut dmx_buffers = vec![[0u8; 512]; dmx_ports.len()];
        loop {
            // Process a control event if one is pending.
            if let Err(err) = self.control(CONTROL_TIMEOUT) {
                error!("A control error occurred: {err:#}.");
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
                self.render(&mut dmx_buffers);
                for (port, buffer) in dmx_ports.iter_mut().zip(&dmx_buffers) {
                    if let Err(e) = port.write(buffer) {
                        error!("DMX write error: {e:#}.");
                    }
                }
            }
        }
    }

    fn control(&mut self, timeout: Duration) -> anyhow::Result<()> {
        let msg = match self.osc_controller.recv(timeout)? {
            Some(m) => m,
            None => {
                return Ok(());
            }
        };

        match msg.msg {
            ControlMessagePayload::Master(mc) => {
                self.master_controls.control(mc, &self.osc_controller);
                Ok(())
            }
            ControlMessagePayload::Animation(msg) => {
                self.animation_ui_state
                    .control(msg, &mut self.patch, &self.osc_controller)
            }
            ControlMessagePayload::RefreshUI => {
                self.master_controls.emit_state(&self.osc_controller);
                for group in self.patch.iter() {
                    group.emit_state(&self.osc_controller);
                }
                self.animation_ui_state
                    .emit_state(&mut self.patch, &self.osc_controller)
            }
            ControlMessagePayload::Fixture(fixture_control_msg) => {
                let Some(group_key) = msg.key.as_ref() else {
                    bail!("no fixture group key was provided with a fixture control message");
                };
                // Identify the correct fixture to handle this message.
                let Some(fixture) = self.patch.get_mut(group_key) else {
                    bail!("no fixture found for key: {:?}", msg.key);
                };

                fixture.control(fixture_control_msg, &self.osc_controller)
            }
            ControlMessagePayload::Error(msg) => {
                bail!("control processing error: {msg}")
            }
        }
    }

    fn update(&mut self, delta_t: Duration) {
        self.master_controls.update(delta_t);
        for fixture in self.patch.iter_mut() {
            fixture.update(delta_t, UnipolarFloat::ZERO);
        }
        if let Some(ref clock_service) = self.clock_service {
            let clock_state = clock_service.get();
            self.master_controls.clock_state = clock_state.clock_bank;
            self.master_controls.audio_envelope = clock_state.audio_envelope;
        }
    }

    fn render(&self, dmx_buffers: &mut [DmxBuffer]) {
        // NOTE: we don't bother to empty the buffer because we will always
        // overwrite all previously-rendered state.
        for group in self.patch.iter() {
            group.render(&self.master_controls, dmx_buffers);
        }
    }
}
