//! Control profile for the "house light" Starlight white laser moonflower.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Default, Debug)]
pub struct Starlight {
    controls: GroupControlMap<ControlMessage>,
    dimmer: UnipolarFloat,
    strobe: GenericStrobe,
    rotation: BipolarFloat,
}

impl PatchAnimatedFixture for Starlight {
    const NAME: FixtureType = FixtureType("Starlight");
    fn channel_count(&self) -> usize {
        4
    }
}

impl Starlight {
    fn handle_state_change(&mut self, sc: StateChange, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        match sc {
            Dimmer(v) => self.dimmer = v,
            Rotation(v) => self.rotation = v,
            Strobe(v) => self.strobe.handle_state_change(v),
        };
        Self::emit(sc, emitter);
    }
}

impl AnimatedFixture for Starlight {
    type Target = AnimationTarget;
    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: &TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        dmx_buf[0] = 255; // DMX mode
        let mut dimmer = self.dimmer.val();
        let mut rotation = self.rotation.val();
        for (val, target) in animation_vals {
            use AnimationTarget::*;
            match target {
                // FIXME: might want to do something nicer for unipolar values
                Rotation => rotation += val,
                Dimmer => dimmer += val,
            }
        }
        dmx_buf[1] = unipolar_to_range(0, 255, UnipolarFloat::new(dimmer));
        dmx_buf[2] = self
            .strobe
            .render_range_with_master(group_controls.strobe(), 0, 10, 255);
        dmx_buf[3] = bipolar_to_split_range(BipolarFloat::new(rotation), 127, 1, 128, 255, 0);
    }
}

impl ControllableFixture for Starlight {
    fn populate_controls(&mut self) {
        Self::map_controls(&mut self.controls);
    }
    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        let Some((ctl, _)) = self.controls.handle(msg)? else {
            return Ok(true);
        };
        self.handle_state_change(ctl, emitter);
        Ok(true)
    }

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        use StateChange::*;
        Self::emit(Dimmer(self.dimmer), emitter);
        Self::emit(Rotation(self.rotation), emitter);
        let mut emit_strobe = |ssc| {
            Self::emit(Strobe(ssc), emitter);
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

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    EnumString,
    EnumIter,
    EnumDisplay,
    FromPrimitive,
    ToPrimitive,
)]
pub enum AnimationTarget {
    #[default]
    Dimmer,
    Rotation,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        matches!(self, Self::Dimmer)
    }
}

impl Starlight {
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        use StateChange::*;
        map.add_unipolar("Dimmer", Dimmer);
        map.add_bipolar("Rotation", |v| Rotation(bipolar_fader_with_detent(v)));
        map_strobe(map, "Strobe", &wrap_strobe);
    }
}

fn wrap_strobe(sc: GenericStrobeStateChange) -> ControlMessage {
    StateChange::Strobe(sc)
}

impl HandleOscStateChange<StateChange> for Starlight {
    fn emit_osc_state_change<S>(_sc: StateChange, _send: &S)
    where
        S: crate::osc::EmitScopedOscMessage + ?Sized,
    {
        // FIXME: implement talkback
    }
}
