//! Clay Paky Astroscan - drunken sailor extraordinaire

use log::error;
use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

use super::strobe::{DimmerStrobe, RenderStrobeToRange, ShutterStrobe, Strobe};

#[derive(Debug)]
struct Active(bool);

impl Default for Active {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Debug)]
pub struct Astroscan {
    lamp_on: BoolChannel,
    shutter: ChannelLevel<DimmerStrobe>,
    iris: UnipolarChannel,
    color: LabeledSelect,
    gobo: IndexedSelectMult,
    gobo_rotation: BipolarSplitChannelMirror,
    mirror_rotation: BipolarSplitChannelMirror,
    pan: BipolarChannelMirror,
    tilt: BipolarChannelMirror,
}

impl Default for Astroscan {
    fn default() -> Self {
        Self {
            lamp_on: Bool::full_channel("LampOn", 2),
            shutter: ChannelLevel::wrap(ShutterStrobe::new(
                Unipolar::channel("Dimmer", 3, 0, 139),
                Strobe::channel("Strobe", 3, 140, 243, 0),
            )),
            iris: Unipolar::full_channel("Iris", 0),
            color: LabeledSelect::new(
                "Color",
                1,
                vec![
                    ("Open", 0),
                    ("Red", 14),
                    ("Yellow", 32),
                    ("Violet", 51),
                    ("Green", 67),
                    ("Orange", 81),
                    ("Blue", 98),
                    ("Pink", 115), // 127 back to white
                ],
            ),
            gobo: IndexedSelect::multiple("Gobo", 6, false, 5, 55),
            gobo_rotation: Bipolar::split_channel("GoboRotation", 7, 189, 128, 193, 255, 191)
                .with_mirroring(true),
            mirror_rotation: Bipolar::split_channel("MirrorRotation", 8, 189, 128, 193, 255, 191)
                .with_mirroring(true),
            pan: Bipolar::channel("Pan", 4, 0, 255).with_mirroring(true),
            tilt: Bipolar::channel("Tilt", 5, 0, 255).with_mirroring(false),
        }
    }
}

impl PatchAnimatedFixture for Astroscan {
    const NAME: FixtureType = FixtureType("Astroscan");
    fn channel_count(&self) -> usize {
        9
    }
}

impl ControllableFixture for Astroscan {
    fn populate_controls(&mut self) {}

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.lamp_on.emit_state(emitter);
        self.shutter.emit_state(emitter);
        self.iris.emit_state(emitter);
        self.color.emit_state(emitter);
        self.gobo.emit_state(emitter);
        self.gobo_rotation.emit_state(emitter);
        self.mirror_rotation.emit_state(emitter);
        self.pan.emit_state(emitter);
        self.tilt.emit_state(emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        if self.lamp_on.control(msg, emitter)? {
            return Ok(true);
        }
        if self.shutter.control(msg, emitter)? {
            return Ok(true);
        }
        if self.iris.control(msg, emitter)? {
            return Ok(true);
        }
        if self.color.control(msg, emitter)? {
            return Ok(true);
        }
        if self.gobo.control(msg, emitter)? {
            return Ok(true);
        }
        if self.gobo_rotation.control(msg, emitter)? {
            return Ok(true);
        }
        if self.mirror_rotation.control(msg, emitter)? {
            return Ok(true);
        }
        if self.pan.control(msg, emitter)? {
            return Ok(true);
        }
        if self.tilt.control(msg, emitter)? {
            return Ok(true);
        }
        Ok(false)
    }
}

impl AnimatedFixture for Astroscan {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        self.iris
            .render(animation_vals.filter(&AnimationTarget::Iris), dmx_buf);
        self.color.render_no_anim(dmx_buf);
        self.lamp_on.render_no_anim(dmx_buf);
        self.shutter.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::Dimmer),
            dmx_buf,
        );
        self.pan.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::Pan),
            dmx_buf,
        );
        self.tilt.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::Tilt),
            dmx_buf,
        );
        self.gobo.render_no_anim(dmx_buf);
        self.gobo_rotation.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::GoboRotation),
            dmx_buf,
        );
        self.mirror_rotation.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::MirrorRotation),
            dmx_buf,
        );
    }
}

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
    Iris,
    GoboRotation,
    MirrorRotation,
    Pan,
    Tilt,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        matches!(self, Self::Dimmer | Self::Iris)
    }
}
