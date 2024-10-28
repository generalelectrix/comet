//! Show-level controls.

use std::time::Duration;

use number::UnipolarFloat;
use tunnels::clock_server::StaticClockBank;

use crate::animation::AnimationUIState;
use crate::channel::{ChannelStateEmitter, Channels};
use crate::control::prelude::*;
use crate::fixture::prelude::*;
use crate::fixture::Patch;
use crate::osc::ScopedControlEmitter;

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

    pub fn emit_state(&self, emitter: &dyn EmitScopedControlMessage) {
        use StateChange::*;
        let mut emit_strobe = |ssc| {
            Self::emit(Strobe(ssc), emitter);
        };
        self.strobe.state.emit_state(&mut emit_strobe);
    }

    pub fn handle_state_change(&mut self, sc: StateChange, emitter: &dyn EmitScopedControlMessage) {
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
                self.handle_state_change(
                    sc,
                    &ScopedControlEmitter {
                        entity: GROUP,
                        emitter,
                    },
                );
            }
            ControlMessage::RefreshUI => {
                self.emit_state(&ScopedControlEmitter {
                    entity: GROUP,
                    emitter,
                });
                channels.emit_state(false, patch, emitter);
                for group in patch.iter() {
                    group.emit_state(ChannelStateEmitter::new(
                        channels.channel_for_fixture(group.key()),
                        emitter,
                    ));
                }
                if let Some(channel) = channels.current_channel() {
                    animation_ui_state.emit_state(
                        channel,
                        channels,
                        patch,
                        &ScopedControlEmitter {
                            entity: crate::osc::animation::GROUP,
                            emitter,
                        },
                    )?;
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

pub(crate) const GROUP: &str = "Master";

const USE_MASTER_STROBE_RATE: Button = button("UseMasterStrobeRate");
const REFRESH_UI: Button = button("RefreshUI");

impl MasterControls {
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        map_strobe(map, "Strobe", &wrap_strobe);
        USE_MASTER_STROBE_RATE.map_state(map, |v| {
            ControlMessage::State(StateChange::UseMasterStrobeRate(v))
        });
        REFRESH_UI.map_trigger(map, || ControlMessage::RefreshUI)
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessage {
    ControlMessage::State(StateChange::Strobe(sc))
}

impl HandleOscStateChange<StateChange> for MasterControls {}
