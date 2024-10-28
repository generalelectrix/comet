//! Martin Wizard Extreme - the one that Goes Slow

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::control::prelude::*;
use crate::fixture::prelude::*;

#[derive(Debug)]
pub struct WizardExtreme {
    shutter: UnipolarChannelLevel<DimmerStrobe>,
    color: LabeledSelect,
    twinkle: Bool<()>,
    twinkle_speed: UnipolarChannel,
    gobo: IndexedSelectMult,
    drum_rotation: BipolarSplitChannelMirror,
    drum_swivel: BipolarChannelMirror,
    reflector_rotation: BipolarSplitChannelMirror,
}

impl Default for WizardExtreme {
    fn default() -> Self {
        Self {
            shutter: ChannelLevel::wrap(ShutterStrobe::new(
                Unipolar::channel("Dimmer", 0, 0, 129),
                Strobe::channel("Strobe", 0, 189, 130, 0),
            )),
            color: LabeledSelect::new(
                "Color",
                2,
                vec![
                    ("Open", 0),
                    ("Blue", 12),
                    ("Orange", 24),
                    ("Purple", 36),
                    ("Green", 48),
                    ("DarkBlue", 60),
                    ("Yellow", 72),
                    ("Magenta", 84),
                ],
            ),
            twinkle: Bool::new_off("Twinkle", ()),
            twinkle_speed: Unipolar::channel("TwinkleSpeed", 2, 176, 243),
            // 14 gobos, including the open position
            gobo: IndexedSelect::multiple("Gobo", 4, false, 14, 12, 0),
            drum_rotation: Bipolar::split_channel("DrumRotation", 7, 2, 63, 127, 66, 0)
                .with_detent()
                .with_mirroring(true),
            drum_swivel: Bipolar::channel("DrumSwivel", 6, 0, 127)
                .with_detent()
                .with_mirroring(true),
            reflector_rotation: Bipolar::split_channel("ReflectorRotation", 1, 2, 63, 127, 66, 0)
                .with_detent()
                .with_mirroring(true),
        }
    }
}

impl PatchAnimatedFixture for WizardExtreme {
    const NAME: FixtureType = FixtureType("WizardExtreme");
    fn channel_count(&self) -> usize {
        11
    }
}

impl ControllableFixture for WizardExtreme {
    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.shutter.emit_state(emitter);
        self.color.emit_state(emitter);
        self.twinkle.emit_state(emitter);
        self.twinkle_speed.emit_state(emitter);
        self.gobo.emit_state(emitter);
        self.drum_rotation.emit_state(emitter);
        self.drum_swivel.emit_state(emitter);
        self.reflector_rotation.emit_state(emitter);
    }

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        if self.shutter.control(msg, emitter)? {
            return Ok(true);
        }
        if self.color.control(msg, emitter)? {
            return Ok(true);
        }
        if self.twinkle.control(msg, emitter)? {
            return Ok(true);
        }
        if self.twinkle_speed.control(msg, emitter)? {
            return Ok(true);
        }
        if self.gobo.control(msg, emitter)? {
            return Ok(true);
        }
        if self.drum_rotation.control(msg, emitter)? {
            return Ok(true);
        }
        if self.drum_swivel.control(msg, emitter)? {
            return Ok(true);
        }
        if self.reflector_rotation.control(msg, emitter)? {
            return Ok(true);
        }
        Ok(false)
    }

    fn control_from_channel(
        &mut self,
        msg: &ChannelControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<()> {
        self.shutter.control_from_channel(msg, emitter)?;
        Ok(())
    }
}

impl AnimatedFixture for WizardExtreme {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        self.shutter.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::Dimmer),
            dmx_buf,
        );
        self.reflector_rotation.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::ReflectorRotation),
            dmx_buf,
        );
        if self.twinkle.val() {
            self.twinkle_speed.render(
                animation_vals.filter(&AnimationTarget::TwinkleSpeed),
                dmx_buf,
            );
        } else {
            self.color.render_no_anim(dmx_buf);
        }
        self.gobo.render_no_anim(dmx_buf);

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
        dmx_buf[3] = 0; // color shake
        dmx_buf[5] = 0; // gobo shake
        dmx_buf[8] = 0;
        dmx_buf[9] = 0;
        dmx_buf[10] = 0;
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
    TwinkleSpeed,
    DrumRotation,
    DrumSwivel,
    ReflectorRotation,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        matches!(self, Self::Dimmer | Self::TwinkleSpeed)
    }
}
