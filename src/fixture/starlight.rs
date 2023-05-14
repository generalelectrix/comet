//! Control profile for the "house light" Starlight white laser moonflower.

use number::{BipolarFloat, UnipolarFloat};

use super::generic::{GenericStrobe, GenericStrobeStateChange};
use super::{
    ControllableFixture, EmitFixtureStateChange as EmitShowStateChange,
    FixtureControlMessage, NonAnimatedFixture, PatchFixture,
};
use crate::master::MasterControls;
use crate::util::bipolar_to_split_range;
use crate::util::unipolar_to_range;

#[derive(Default, Debug)]
pub struct Starlight {
    dimmer: UnipolarFloat,
    strobe: GenericStrobe,
    rotation: BipolarFloat,
}

impl PatchFixture for Starlight {
    const NAME: &'static str = "starlight";
    fn channel_count(&self) -> usize {
        4
    }
}

impl Starlight {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &mut dyn EmitShowStateChange) {
        use StateChange::*;
        match sc {
            Dimmer(v) => self.dimmer = v,
            Rotation(v) => self.rotation = v,
            Strobe(v) => self.strobe.handle_state_change(v),
        };
        emitter.emit_starlight(sc);
    }
}

impl NonAnimatedFixture for Starlight {
    fn render(&self, master_controls: &MasterControls, dmx_buf: &mut [u8]) {
        dmx_buf[0] = 255; // DMX mode
        dmx_buf[1] = unipolar_to_range(0, 255, self.dimmer);
        dmx_buf[2] = self
            .strobe
            .render_range_with_master(master_controls.strobe(), 0, 10, 255);
        dmx_buf[3] = bipolar_to_split_range(self.rotation, 0, 127, 255, 128, 0);
    }
}

impl ControllableFixture for Starlight {
    fn control(
        &mut self,
        msg: FixtureControlMessage,
        emitter: &mut dyn EmitShowStateChange,
    ) -> Option<FixtureControlMessage> {
        match msg {
            FixtureControlMessage::Starlight(msg) => {
                self.handle_state_change(msg, emitter);
                None
            }
            other => Some(other),
        }
    }

    fn emit_state(&self, emitter: &mut dyn EmitShowStateChange) {
        use StateChange::*;
        emitter.emit_starlight(Dimmer(self.dimmer));
        emitter.emit_starlight(Rotation(self.rotation));
        let mut emit_strobe = |ssc| {
            emitter.emit_starlight(Strobe(ssc));
        };
        self.strobe.emit_state(&mut emit_strobe);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StateChange {
    Dimmer(UnipolarFloat),
    Strobe(GenericStrobeStateChange),
    Rotation(BipolarFloat),
}

// Starlight has no controls that are not represented as state changes.
pub type ControlMessage = StateChange;
