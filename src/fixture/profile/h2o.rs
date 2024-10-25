//! Intuitive control profile for the American DJ H2O DMX Pro.

use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

use crate::fixture::prelude::*;
use crate::osc::prelude::*;

#[derive(Debug)]
pub struct H2O {
    dimmer: UnipolarChannelLevel<UnipolarChannel>,
    rotation: BipolarSplitChannelMirror,
    fixed_color: LabeledSelect,
    color_rotate: Bool<()>,
    color_rotation: BipolarSplitChannel,
}

impl Default for H2O {
    fn default() -> Self {
        Self {
            dimmer: Unipolar::full_channel("Dimmer", 0).with_channel_level(),
            rotation: Bipolar::split_channel("Rotation", 1, 120, 10, 135, 245, 0)
                .with_mirroring(true),
            fixed_color: LabeledSelect::new(
                "FixedColor",
                2,
                vec![
                    ("White", 0),
                    ("WhiteOrange", 11),
                    ("Orange", 22),
                    ("OrangeGreen", 33),
                    ("Green", 44),
                    ("GreenBlue", 55),
                    ("Blue", 66),
                    ("BlueYellow", 77),
                    ("Yellow", 88),
                    ("YellowPurple", 99),
                    ("Purple", 110),
                    ("PurpleWhite", 121),
                ],
            ),
            color_rotate: Bool::new_off("ColorRotate", ()),
            color_rotation: Bipolar::split_channel("ColorRotation", 2, 186, 128, 197, 255, 187),
        }
    }
}

impl PatchAnimatedFixture for H2O {
    const NAME: FixtureType = FixtureType("H2O");
    fn channel_count(&self) -> usize {
        3
    }
}

impl AnimatedFixture for H2O {
    type Target = AnimationTarget;

    fn render_with_animations(
        &self,
        group_controls: &FixtureGroupControls,
        animation_vals: TargetedAnimationValues<Self::Target>,
        dmx_buf: &mut [u8],
    ) {
        self.dimmer.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::Dimmer),
            dmx_buf,
        );
        self.rotation.render_with_group(
            group_controls,
            animation_vals.filter(&AnimationTarget::Rotation),
            dmx_buf,
        );
        if self.color_rotate.val() {
            self.color_rotation.render_with_group(
                group_controls,
                animation_vals.filter(&AnimationTarget::ColorRotation),
                dmx_buf,
            );
        } else {
            self.fixed_color.render_no_anim(dmx_buf);
        }
    }
}

impl ControllableFixture for H2O {
    

    fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        if self.dimmer.control(msg, emitter)? {
            return Ok(true);
        }
        if self.rotation.control(msg, emitter)? {
            return Ok(true);
        }
        if self.fixed_color.control(msg, emitter)? {
            return Ok(true);
        }
        if self.color_rotate.control(msg, emitter)? {
            return Ok(true);
        }
        if self.color_rotation.control(msg, emitter)? {
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

    fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.dimmer.emit_state(emitter);
        self.rotation.emit_state(emitter);
        self.fixed_color.emit_state(emitter);
        self.color_rotate.emit_state(emitter);
        self.color_rotation.emit_state(emitter);
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
    Rotation,
    ColorRotation,
}

impl AnimationTarget {
    /// Return true if this target is unipolar instead of bipolar.
    #[allow(unused)]
    pub fn is_unipolar(&self) -> bool {
        matches!(self, Self::Dimmer)
    }
}
