//! Show-level controls.

use std::time::Duration;

use number::UnipolarFloat;
use tunnels::clock_server::StaticClockBank;

use crate::animation::AnimationUIState;
use crate::channel::{ChannelStateEmitter, Channels};
use crate::fixture::Patch;
use crate::osc::{GroupControlMap, HandleStateChange, OscControlMessage};
use crate::{
    fixture::generic::{GenericStrobe, GenericStrobeStateChange},
    osc::EmitControlMessage,
};

pub use crate::fixture::FixtureGroupControls;

pub struct MasterControls {
    strobe: Strobe,
    pub clock_state: StaticClockBank,
    pub audio_envelope: UnipolarFloat,
    controls: GroupControlMap<ControlMessage>,
}

impl MasterControls {
    pub fn new() -> Self {
        let mut controls = GroupControlMap::default();
        Self::map_controls(&mut controls);
        Self {
            strobe: Default::default(),
            clock_state: Default::default(),
            audio_envelope: Default::default(),
            controls,
        }
    }

    pub fn strobe(&self) -> &Strobe {
        &self.strobe
    }

    pub fn update(&mut self, _delta_t: Duration) {}

    pub fn emit_state(&self, emitter: &dyn EmitControlMessage) {
        use StateChange::*;
        let mut emit_strobe = |ssc| {
            Self::emit(Strobe(ssc), emitter);
        };
        self.strobe.state.emit_state(&mut emit_strobe);
    }

    pub fn handle_state_change(&mut self, sc: StateChange, emitter: &dyn EmitControlMessage) {
        use StateChange::*;
        match sc {
            Strobe(sc) => self.strobe.state.handle_state_change(sc),
            UseMasterStrobeRate(v) => self.strobe.use_master_rate = v,
        }
        Self::emit(sc, emitter);
    }

    // FIXME: we should lift UI refresh up and out of here
    pub fn control(
        &mut self,
        msg: &OscControlMessage,
        channels: &Channels,
        patch: &Patch,
        animation_ui_state: &AnimationUIState,
        emitter: &dyn EmitControlMessage,
    ) -> anyhow::Result<()> {
        let Some((ctl, _)) = self.controls.handle(msg)? else {
            return Ok(());
        };
        match ctl {
            ControlMessage::State(sc) => {
                self.handle_state_change(sc, emitter);
            }
            ControlMessage::RefreshUI => {
                self.emit_state(emitter);
                channels.emit_state(false, patch, emitter);
                for group in patch.iter() {
                    group.emit_state(ChannelStateEmitter::new(
                        channels.channel_for_fixture(group.key()),
                        emitter,
                    ));
                }
                if let Some(channel) = channels.current_channel() {
                    animation_ui_state.emit_state(channel, channels, patch, emitter)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ControlMessage {
    State(StateChange),
    RefreshUI,
}

#[derive(Debug, Clone)]
pub enum StateChange {
    Strobe(GenericStrobeStateChange),
    UseMasterStrobeRate(bool),
}

#[derive(Debug, Default)]
pub struct Strobe {
    pub state: GenericStrobe,
    pub use_master_rate: bool,
}
