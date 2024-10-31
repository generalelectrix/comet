//! Control decorators to bind OSC controls to channel controls.

use std::marker::PhantomData;

use number::{BipolarFloat, UnipolarFloat};

use crate::{
    channel::{ChannelControlMessage, ChannelStateChange, KnobIndex, KnobValue},
    osc::{FixtureStateEmitter, OscControlMessage},
};

use super::{OscControl, RenderToDmxWithAnimations};

#[derive(Debug)]
pub struct ChannelControl<C, T, H>
where
    C: OscControl<T> + RenderToDmxWithAnimations,
    H: ChannelHandler<T>,
{
    pub control: C,
    label: String,
    handler: H,
    phantom: PhantomData<T>,
}

impl<C, T, H> ChannelControl<C, T, H>
where
    C: OscControl<T> + RenderToDmxWithAnimations,
    H: ChannelHandler<T>,
{
    pub fn wrap(control: C, label: String, handler: H) -> Self {
        Self {
            control,
            label,
            handler,
            phantom: PhantomData,
        }
    }

    /// Control this channel-control-wrapped control from OSC.
    pub fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        self.control
            .control_with_callback(msg, emitter, |v| emitter.emit_channel(self.handler.emit(v)))
    }

    /// Emit state for this channel-wrapped control, including for the channel control.
    pub fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.control
            .emit_state_with_callback(emitter, |v| emitter.emit_channel(self.handler.emit(v)));
    }

    /// Handle a channel-level control.
    pub fn control_from_channel(
        &mut self,
        msg: &ChannelControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        let Some(v) = self.handler.parse(msg) else {
            return Ok(false);
        };
        self.control.control_direct(v, emitter)?;
        // Echo the exact channel message back out.
        emitter.emit_channel(*msg);
        Ok(true)
    }
}

/// Something that adapts channel control messages to a concrete type T.
pub trait ChannelHandler<T> {
    /// Parse a specific channel control message into a T.
    /// This will usually handle a specific channel control message.
    fn parse(&self, msg: &ChannelControlMessage) -> Option<T>;

    /// Handle converting T into a specific channel state change.
    fn emit(&self, val: &T) -> ChannelStateChange;
}

/// Channel level control.
#[derive(Debug)]
pub struct ChannelLevelHandler;

impl ChannelHandler<UnipolarFloat> for ChannelLevelHandler {
    fn parse(&self, msg: &ChannelControlMessage) -> Option<UnipolarFloat> {
        let ChannelControlMessage::Level(v) = msg else {
            return None;
        };
        Some(*v)
    }

    fn emit(&self, val: &UnipolarFloat) -> ChannelStateChange {
        ChannelStateChange::Level(*val)
    }
}

pub type ChannelLevelControl<C, T> = ChannelControl<C, T, ChannelLevelHandler>;

pub type ChannelLevelUnipolar<C> = ChannelLevelControl<C, UnipolarFloat>;

/// Channel level control for bool controls, like on/off shutters or non-dimmable lasers.
impl ChannelHandler<bool> for ChannelLevelHandler {
    fn parse(&self, msg: &ChannelControlMessage) -> Option<bool> {
        let ChannelControlMessage::Level(v) = msg else {
            return None;
        };
        Some(*v > 0.5)
    }

    fn emit(&self, val: &bool) -> ChannelStateChange {
        ChannelStateChange::Level(if *val {
            UnipolarFloat::ONE
        } else {
            UnipolarFloat::ZERO
        })
    }
}

pub type ChannelLevelBool<C> = ChannelLevelControl<C, bool>;

/// Delegate rendering to the inner control.
impl<C, T, H> RenderToDmxWithAnimations for ChannelControl<C, T, H>
where
    C: OscControl<T> + RenderToDmxWithAnimations,
    H: ChannelHandler<T>,
{
    fn render(&self, animations: impl Iterator<Item = f64>, dmx_buf: &mut [u8]) {
        self.control.render(animations, dmx_buf);
    }
    fn render_no_anim(&self, dmx_buf: &mut [u8]) {
        self.control.render_no_anim(dmx_buf);
    }
    fn render_with_group(
        &self,
        group_controls: &crate::fixture::FixtureGroupControls,
        animations: impl Iterator<Item = f64>,
        dmx_buf: &mut [u8],
    ) {
        self.control
            .render_with_group(group_controls, animations, dmx_buf);
    }
}

/// Channel knob control.
#[derive(Debug)]
pub struct ChannelKnobHandler {
    pub index: KnobIndex,
}

impl ChannelKnobHandler {
    /// Return the knob value if this channel control message is for this handler.
    fn matches<'a>(&self, msg: &'a ChannelControlMessage) -> Option<&'a KnobValue> {
        let ChannelControlMessage::Knob { index, value } = msg else {
            return None;
        };
        if *index != self.index {
            return None;
        }
        Some(value)
    }
}

impl ChannelHandler<UnipolarFloat> for ChannelKnobHandler {
    fn parse(&self, msg: &ChannelControlMessage) -> Option<UnipolarFloat> {
        Some(self.matches(msg)?.as_unipolar())
    }

    fn emit(&self, val: &UnipolarFloat) -> ChannelStateChange {
        ChannelStateChange::Knob {
            index: self.index,
            value: KnobValue::Unipolar(*val),
        }
    }
}

impl ChannelHandler<BipolarFloat> for ChannelKnobHandler {
    fn parse(&self, msg: &ChannelControlMessage) -> Option<BipolarFloat> {
        Some(self.matches(msg)?.as_bipolar())
    }

    fn emit(&self, val: &BipolarFloat) -> ChannelStateChange {
        ChannelStateChange::Knob {
            index: self.index,
            value: KnobValue::Bipolar(*val),
        }
    }
}

pub type ChannelKnobControl<C, T> = ChannelControl<C, T, ChannelKnobHandler>;

pub type ChannelKnobUnipolar<C> = ChannelKnobControl<C, UnipolarFloat>;

pub type ChannelKnobBipolar<C> = ChannelKnobControl<C, BipolarFloat>;
