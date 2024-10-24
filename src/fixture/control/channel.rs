//! Control decorators to bind OSC controls to channel-level controls.

use number::UnipolarFloat;

use crate::{
    channel::{ChannelControlMessage, ChannelStateChange},
    osc::{FixtureStateEmitter, OscControlMessage},
};

use super::{OscControl, RenderToDmxWithAnimations};

#[derive(Debug)]
pub struct ChannelLevel<C>
where
    C: OscControl<UnipolarFloat> + RenderToDmxWithAnimations,
{
    pub control: C,
}

impl<C> ChannelLevel<C>
where
    C: OscControl<UnipolarFloat> + RenderToDmxWithAnimations,
{
    pub fn wrap(control: C) -> Self {
        Self { control }
    }

    /// Control this channel-control-wrapped control from OSC.
    pub fn control(
        &mut self,
        msg: &OscControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        self.control.control_with_callback(msg, emitter, |v| {
            emitter.emit_channel(ChannelStateChange::Level(*v))
        })
    }

    /// Emit state for this channel-wrapped control, including for the channel control.
    pub fn emit_state(&self, emitter: &FixtureStateEmitter) {
        self.control.emit_state_with_callback(emitter, |v| {
            emitter.emit_channel(ChannelStateChange::Level(*v))
        });
    }

    /// Handle a channel-level control.
    pub fn control_from_channel(
        &mut self,
        msg: &ChannelControlMessage,
        emitter: &FixtureStateEmitter,
    ) -> anyhow::Result<bool> {
        #[allow(irrefutable_let_patterns)]
        let ChannelControlMessage::Level(v) = msg
        else {
            return Ok(false);
        };
        self.control.control_direct(*v, emitter)?;
        emitter.emit_channel(ChannelStateChange::Level(*v));
        Ok(true)
    }
}

impl<C> RenderToDmxWithAnimations for ChannelLevel<C>
where
    C: OscControl<UnipolarFloat> + RenderToDmxWithAnimations,
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
