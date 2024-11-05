//! Show-level controls.

use std::time::Duration;

use number::UnipolarFloat;
use tunnels::clock_server::StaticClockBank;

use crate::fixture::prelude::*;
use crate::osc::ScopedControlEmitter;

pub struct MasterControls {
    strobe_on: Bool<()>,
    strobe_rate: Unipolar<()>,
    use_master_rate: Bool<()>,
    pub clock_state: StaticClockBank,
    pub audio_envelope: UnipolarFloat,
}

impl MasterControls {
    pub fn new() -> Self {
        Self {
            strobe_on: Bool::new_off("StrobeOn", ()),
            strobe_rate: Unipolar::new("StrobeRate", ()),
            use_master_rate: Bool::new_off("UseMasterStrobeRate", ()),
            clock_state: Default::default(),
            audio_envelope: Default::default(),
        }
    }

    pub fn strobe(&self) -> Strobe {
        Strobe {
            on: self.strobe_on.val(),
            rate: self.strobe_rate.val(),
            use_master_rate: self.use_master_rate.val(),
        }
    }

    pub fn update(&mut self, _delta_t: Duration) {}

    pub fn emit_state(&self, emitter: &dyn EmitControlMessage) {
        let scoped_emitter = &ScopedControlEmitter {
            entity: GROUP,
            emitter,
        };
        self.strobe_on
            .emit_state_with_callback(scoped_emitter, |v| {
                emitter.emit_midi_master_message(&StateChange::StrobeOn(*v));
            });
        self.strobe_rate
            .emit_state_with_callback(scoped_emitter, |v| {
                emitter.emit_midi_master_message(&StateChange::StrobeRate(*v));
            });
        self.strobe_on
            .emit_state_with_callback(scoped_emitter, |v| {
                emitter.emit_midi_master_message(&StateChange::UseMasterStrobeRate(*v));
            });
    }

    pub fn control(
        &mut self,
        msg: &ControlMessage,
        emitter: &dyn EmitControlMessage,
    ) -> anyhow::Result<()> {
        let scoped_emitter = &ScopedControlEmitter {
            entity: GROUP,
            emitter,
        };

        match msg {
            StateChange::StrobeOn(v) => {
                self.strobe_on.control_direct(*v, scoped_emitter)?;
            }
            StateChange::StrobeRate(v) => {
                self.strobe_rate.control_direct(*v, scoped_emitter)?;
            }
            StateChange::UseMasterStrobeRate(v) => {
                self.use_master_rate.control_direct(*v, scoped_emitter)?;
            }
        }

        emitter.emit_midi_master_message(msg);
        Ok(())
    }

    pub fn control_osc(
        &mut self,
        msg: &OscControlMessage,
        emitter: &dyn EmitControlMessage,
    ) -> anyhow::Result<()> {
        let scoped_emitter = &ScopedControlEmitter {
            entity: GROUP,
            emitter,
        };
        if self.strobe_on.control(msg, scoped_emitter)? {
            emitter.emit_midi_master_message(&StateChange::StrobeOn(self.strobe_on.val()));
            return Ok(());
        }
        if self.strobe_rate.control(msg, scoped_emitter)? {
            emitter.emit_midi_master_message(&StateChange::StrobeRate(self.strobe_rate.val()));
            return Ok(());
        }
        if self.use_master_rate.control(msg, scoped_emitter)? {
            emitter.emit_midi_master_message(&StateChange::UseMasterStrobeRate(
                self.use_master_rate.val(),
            ));
            return Ok(());
        }
        Ok(())
    }
}

pub type ControlMessage = StateChange;

#[derive(Debug, Clone)]
pub enum StateChange {
    StrobeOn(bool),
    StrobeRate(UnipolarFloat),
    UseMasterStrobeRate(bool),
}

#[derive(Debug, Default)]
pub struct Strobe {
    pub on: bool,
    pub rate: UnipolarFloat,
    pub use_master_rate: bool,
}

pub const GROUP: &str = "Master";
