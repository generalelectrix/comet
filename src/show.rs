use std::time::{Duration, Instant};

use crate::{
    animation::AnimationUIState,
    channel::{ChannelStateEmitter, Channels},
    clock_service::ClockService,
    config::Config,
    dmx::DmxBuffer,
    fixture::{FixtureGroup, FixtureGroupKey, GroupName, Patch},
    master::MasterControls,
    osc::{ControlMessageType, OscController},
};

pub use crate::channel::ChannelId;
use anyhow::{bail, Result};
use log::error;
use number::UnipolarFloat;
use rust_dmx::DmxPort;

pub struct Show {
    osc_controller: OscController,
    patch: Patch,
    channels: Channels,
    master_controls: MasterControls,
    animation_ui_state: AnimationUIState,
    clock_service: Option<ClockService>,
}

const CONTROL_TIMEOUT: Duration = Duration::from_millis(1);
const UPDATE_INTERVAL: Duration = Duration::from_millis(20);

impl Show {
    pub fn new(cfg: Config, clock_service: Option<ClockService>) -> Result<Self> {
        let mut channels = Channels::new();
        let mut patch = Patch::default();

        let osc_controller = OscController::new(cfg.receive_port, cfg.controllers)?;

        for fixture in cfg.fixtures.into_iter() {
            patch.patch(&mut channels, fixture)?;
        }

        for group in patch.iter() {
            group.emit_state(ChannelStateEmitter::new(
                channels.channel_for_fixture(group.key()),
                &osc_controller.sender_with_metadata(None),
            ));
        }

        let master_controls = MasterControls::new();
        master_controls.emit_state(&osc_controller.sender_with_metadata(None));

        let initial_channel = channels.validate_channel(0).ok();

        channels.emit_state(false, &patch, &osc_controller.sender_with_metadata(None));

        let animation_ui_state = AnimationUIState::new(initial_channel);

        // Configure animation controls if we have at least one animated fixture.
        if patch.iter().any(FixtureGroup::is_animated) {
            animation_ui_state.emit_state(
                initial_channel.unwrap(),
                &channels,
                &patch,
                &osc_controller.sender_with_metadata(None),
            )?;
        };

        Ok(Self {
            osc_controller,
            patch,
            channels,
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

        let sender = self
            .osc_controller
            .sender_with_metadata(Some(&msg.client_id));

        match ControlMessageType::parse(msg.entity_type()) {
            ControlMessageType::Master => self.master_controls.control(
                &msg,
                &self.channels,
                &self.patch,
                &self.animation_ui_state,
                &sender,
            ),
            ControlMessageType::Channel => self.channels.control(&msg, &mut self.patch, &sender),
            ControlMessageType::Animation => {
                let Some(channel) = self.channels.current_channel() else {
                    bail!("cannot handle animation control message because no channel is selected\n{msg:?}");
                };
                self.animation_ui_state.control(
                    &msg,
                    channel,
                    &self.channels,
                    &mut self.patch,
                    &sender,
                )
            }
            ControlMessageType::Fixture => {
                let Some(fixture_type) = self.patch.lookup_fixture_type(msg.entity_type()) else {
                    bail!(
                        "entity type \"{}\" not registered with patch, from OSC message {msg:?}",
                        msg.entity_type()
                    );
                };
                let group_key = FixtureGroupKey {
                    fixture: fixture_type,
                    group: msg.group().map(GroupName::new),
                };
                self.patch.get_mut(&group_key)?.control(
                    &msg,
                    ChannelStateEmitter::new(
                        self.channels.channel_for_fixture(&group_key),
                        &sender,
                    ),
                )
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
