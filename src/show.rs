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
    osc::{EmitControlMessage, HandleStateChange, OscController, TalkbackMode},
};

use anyhow::{bail, Result};
use log::error;
use number::UnipolarFloat;
use rust_dmx::DmxPort;
use serde::Deserialize;

pub struct Show {
    osc_controller: OscController,
    patch: Patch,
    master_controls: MasterControls,
    show_ui_state: ShowUIState,
    animation_ui_state: AnimationUIState,
    clock_service: Option<ClockService>,
}

const CONTROL_TIMEOUT: Duration = Duration::from_millis(1);
const UPDATE_INTERVAL: Duration = Duration::from_millis(20);

impl Show {
    pub fn new(cfg: Config, clock_service: Option<ClockService>) -> Result<Self> {
        let mut patch = Patch::default();

        let mut osc_controller = OscController::new(cfg.receive_port, cfg.controllers)?;

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

            group.emit_state(&osc_controller.sender_with_metadata(None, TalkbackMode::All));
        }

        let master_controls = MasterControls::default();
        osc_controller.map_controls(&master_controls);
        master_controls.emit_state(&osc_controller.sender_with_metadata(None, TalkbackMode::All));

        let initial_channel = patch.validate_channel(0).ok();

        let show_ui_state = ShowUIState {
            current_channel: initial_channel,
        };
        osc_controller.map_controls(&show_ui_state);
        show_ui_state.emit_state(
            &mut patch,
            &osc_controller.sender_with_metadata(None, TalkbackMode::All),
        );

        let animation_ui_state = AnimationUIState::new(initial_channel);

        // Configure animation controls if we have at least one animated fixture.
        if patch.iter().any(FixtureGroup::is_animated) {
            osc_controller.map_controls(&animation_ui_state);
            animation_ui_state.emit_state(
                initial_channel.unwrap(),
                &mut patch,
                &osc_controller.sender_with_metadata(None, TalkbackMode::All),
            )?;
        };

        Ok(Self {
            patch,
            osc_controller,
            master_controls,
            show_ui_state,
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
            .sender_with_metadata(Some(&msg.sender_id), msg.talkback);

        match msg.msg {
            ControlMessagePayload::Master(mc) => {
                self.master_controls.control(mc, &sender);
                Ok(())
            }
            ControlMessagePayload::Animation(msg) => {
                let Some(channel) = self.show_ui_state.current_channel else {
                    bail!("cannot handle animation control message because no channel is selected\n{msg:?}");
                };
                self.animation_ui_state
                    .control(msg, channel, &mut self.patch, &sender)
            }
            ControlMessagePayload::Show(msg) => {
                self.show_ui_state.control(msg, &mut self.patch, &sender)
            }
            ControlMessagePayload::RefreshUI => {
                self.master_controls.emit_state(&sender);
                self.show_ui_state.emit_state(&mut self.patch, &sender);
                for group in self.patch.iter() {
                    group.emit_state(&sender);
                }
                if let Some(channel) = self.show_ui_state.current_channel {
                    self.animation_ui_state
                        .emit_state(channel, &mut self.patch, &sender)?;
                }
                Ok(())
            }
            ControlMessagePayload::Fixture(fixture_control_msg) => {
                let Some(group_key) = msg.key.as_ref() else {
                    bail!("no fixture group key was provided with a fixture control message");
                };
                // Identify the correct fixture to handle this message.
                let Some(fixture) = self.patch.get_mut(group_key) else {
                    bail!("no fixture found for key: {:?}", msg.key);
                };

                fixture.control(fixture_control_msg, &sender)
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

/// Which channel is currently selected.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct ChannelId(pub usize);

pub struct ShowUIState {
    current_channel: Option<ChannelId>,
}

impl ShowUIState {
    /// Emit all current animation state, including target and selection.
    pub fn emit_state(&self, patch: &mut Patch, emitter: &dyn EmitControlMessage) {
        if let Some(channel) = self.current_channel {
            Self::emit(StateChange::SelectChannel(channel), emitter);
        }
        Self::emit(
            StateChange::ChannelLabels(patch.channel_labels().collect()),
            emitter,
        );
    }

    /// Handle a control message.
    pub fn control(
        &mut self,
        msg: ControlMessage,
        patch: &mut Patch,
        emitter: &dyn EmitControlMessage,
    ) -> anyhow::Result<()> {
        match msg {
            ControlMessage::SelectChannel(g) => {
                // Validate the channel.
                let channel = patch.validate_channel(g)?;
                if self.current_channel == Some(channel) {
                    // Channel is not changed, ignore.
                    return Ok(());
                }
                self.current_channel = Some(channel);
                self.emit_state(patch, emitter);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum ControlMessage {
    SelectChannel(usize),
}

#[derive(Clone, Debug)]
pub enum StateChange {
    SelectChannel(ChannelId),
    ChannelLabels(Vec<String>),
}
