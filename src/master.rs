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
    /// True if the master strobe clock has flashed on the current update.
    flash: bool,
    /// Strobe clock.
    last_flash_age: Duration,
    pub clock_state: StaticClockBank,
    pub audio_envelope: UnipolarFloat,
}

impl MasterControls {
    pub fn new() -> Self {
        Self {
            strobe_on: Bool::new_off("StrobeOn", ()),
            strobe_rate: Unipolar::new("StrobeRate", ()),
            use_master_rate: Bool::new_off("UseMasterStrobeRate", ()),
            flash: false,
            last_flash_age: Duration::ZERO,
            clock_state: Default::default(),
            audio_envelope: Default::default(),
        }
    }

    pub fn strobe(&self) -> Strobe {
        Strobe {
            on: self.strobe_on.val(),
            rate: self.strobe_rate.val(),
            flash: self.flash,
            use_master_rate: self.use_master_rate.val(),
        }
    }

    pub fn update(&mut self, delta_t: Duration) {
        self.last_flash_age += delta_t;
        if self.strobe_on.val() {
            let interval = strobe_interval_from_rate(self.strobe_rate.val());
            if self.last_flash_age >= interval {
                self.last_flash_age = Duration::ZERO;
                self.flash = true;
            }
        } else {
            self.flash = false;
        }
    }

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
    /// If true, flash on this update.
    pub flash: bool,
}

pub const GROUP: &str = "Master";

/// Convert a unipolar control parameter into a strobe rate.
/// Scaled so the slowest strobing is once per second.
///
/// Values are coerced to be exact multiples of the show update rate.
/// FIXME: this should depend on the show framerate explicitly rather than
/// assuming the show is running at 50Hz.
pub fn strobe_interval_from_rate(rate: UnipolarFloat) -> Duration {
    // lowest rate: 1 flash/sec => 1 sec interval
    // highest rate: 50 flash/sec => 20 ms interval
    // use exact frame intervals
    let raw_interval = (100. / (rate.val() + 0.09)) as u64 - 70;
    // Divide/multiple by the update interval to force into an integer number.
    let coerced_interval = ((raw_interval / 20) * 20).max(20);
    Duration::from_millis(coerced_interval)
}
