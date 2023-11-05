use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use crate::{
    animation::AnimationUIState,
    clock_service::ClockService,
    config::Config,
    fixture::{FixtureControlMessage, FixtureGroup, Patch},
    master::MasterControls,
    osc::{AnimationControls, OscController},
};

use anyhow::Result;
use log::{error, warn};
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
                patched_controls.insert(group.fixture_type().to_string());
            }

            group.emit_state(&mut osc_controller);
        }

        let master_controls = MasterControls::default();
        osc_controller.map_controls(&master_controls);
        master_controls.emit_state(&mut osc_controller);

        // Configure animation controls.
        osc_controller.map_controls(&AnimationControls);
        let animation_ui_state = if patch.iter().any(FixtureGroup::is_animated) {
            let state = AnimationUIState::new(Some(patch.validate_selector(0)?));
            state.emit_state(&mut patch, &mut osc_controller)?;
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

    /// Run the show forever in the current thread.
    pub fn run(&mut self, mut dmx_port: Box<dyn DmxPort>) {
        let mut last_update = Instant::now();
        let mut dmx_buffer = vec![0u8; 512];
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
                self.render(&mut dmx_buffer);
                if let Err(e) = dmx_port.write(&dmx_buffer) {
                    error!("DMX write error: {e:#}.");
                }
            }
        }
    }

    fn control(&mut self, timeout: Duration) -> Result<()> {
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

        if let FixtureControlMessage::Animation(msg) = msg.msg {
            return self
                .animation_ui_state
                .control(msg, &mut self.patch, &mut self.osc_controller);
        }

        if let FixtureControlMessage::RefreshUI = msg.msg {
            self.master_controls.emit_state(&mut self.osc_controller);
            for group in self.patch.iter() {
                group.emit_state(&mut self.osc_controller);
            }
            self.animation_ui_state
                .emit_state(&mut self.patch, &mut self.osc_controller)?;
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
            fixture.update(delta_t, UnipolarFloat::ZERO);
        }
        if let Some(ref clock_service) = self.clock_service {
            let clock_state = clock_service.get();
            self.master_controls.clock_state = clock_state.clock_bank;
            self.master_controls.audio_envelope = clock_state.audio_envelope;
        }
    }

    fn render(&self, dmx_buffer: &mut [u8]) {
        // NOTE: we don't bother to empty the buffer because we will always
        // overwrite all previously-rendered state.
        for group in self.patch.iter() {
            group.render(&self.master_controls, dmx_buffer);
        }
    }
}
