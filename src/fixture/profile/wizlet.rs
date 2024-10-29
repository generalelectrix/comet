//! Control profile for the American DJ (Eliminator) Vortex, aka the Wizlet.
use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::control::prelude::*;
use crate::fixture::prelude::*;

#[derive(Debug)]
pub struct Wizlet {
    drum_swivel: BipolarChannelMirror,
    drum_rotation: BipolarSplitChannelMirror,
    gobo: LabeledSelect,
    reflector_rotation: BipolarSplitChannelMirror,
    strobe: StrobeChannel,
    dimmer: UnipolarChannelLevel<UnipolarChannel>,
}

impl Default for Wizlet {
    fn default() -> Self {
        Self {
            drum_swivel: Bipolar::channel("DrumSwivel", 0, 0, 255)
                .with_detent()
                .with_mirroring(true),
            drum_rotation: Bipolar::split_channel("DrumRotation", 1, 120, 10, 135, 245, 0)
                .with_detent()
                .with_mirroring(true),
            gobo: LabeledSelect::new(
                "Gobo",
                2,
                vec![
                    ("Open", 0),
                    ("RedTriBar", 8),
                    ("BlueHazard", 14),
                    ("GreenTriangle", 20),
                    ("YellowShatter", 26),
                    ("RGBYQuadDot", 32),
                    ("MagentaSquare", 38),
                    ("AquaStar", 44),
                    ("LimeDots", 50),
                    ("WhiteDots", 56),
                ],
            ),
            reflector_rotation: Bipolar::split_channel(
                "ReflectorRotation",
                3,
                120,
                10,
                135,
                245,
                0,
            )
            .with_detent()
            .with_mirroring(true),
            strobe: Strobe::channel("Strobe", 4, 64, 95, 32),
            dimmer: Unipolar::full_channel("Dimmer", 5).with_channel_level(),
        }
    }
}

impl PatchAnimatedFixture for Wizlet {
    const NAME: FixtureType = FixtureType("Wizlet");
    fn channel_count(&self) -> usize {
        12
    }
}

impl ControllableFixture for Wizlet {
    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.drum_swivel.emit_state(emitter);
        self.drum_rotation.emit_state(emitter);
        self.gobo.emit_state(emitter);
        self.reflector_rotation.emit_state(emitter);
        self.strobe.emit_state(emitter);
        self.dimmer.emit_state(emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        if self.drum_swivel.control(msg, emitter)? {
            return Ok(true);
        }
        if self.drum_rotation.control(msg, emitter)? {
            return Ok(true);
        }
        if self.gobo.control(msg, emitter)? {
            return Ok(true);
        }
        if self.reflector_rotation.control(msg, emitter)? {
            return Ok(true);
        }
        if self.strobe.control(msg, emitter)? {
            return Ok(true);
        }
        if self.dimmer.control(msg, emitter)? {
            return Ok(true);
        }
        Ok(false)
    }

    fn control_from_channel(
        &mut self,
        msg: &ChannelControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        self.dimmer.control_from_channel(msg, emitter)?;
        Ok(())
    }
}

impl AnimatedFixture for Wizlet {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        self.drum_swivel.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::DrumSwivel),
            dmx_buf,
        );
        self.drum_rotation.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::DrumRotation),
            dmx_buf,
        );
        self.gobo.render_no_anim(dmx_buf);
        self.reflector_rotation.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::ReflectorRotation),
            dmx_buf,
        );
        self.strobe
            .render_with_group(group_controls, std::iter::empty(), dmx_buf);
        self.dimmer.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::Dimmer),
            dmx_buf,
        );
        dmx_buf[6] = 0; // show
        dmx_buf[7] = 0; // show speed
        dmx_buf[8] = 21; // standard dim mode, overriding whatever is configured
        dmx_buf[9] = 0; // no dimming interpolation
        dmx_buf[10] = 0; // fast pan speed
        dmx_buf[11] = 0; // special; note this can trigger remote fixture reset, might be useful to implement this if they get out of whack
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
    DrumRotation,
    DrumSwivel,
    ReflectorRotation,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        matches!(self, Self::Dimmer)
    }
}
