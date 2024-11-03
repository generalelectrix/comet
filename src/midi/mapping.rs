//! Mappings between show control events and midi device-specific actions.
//!
use tunnels::midi_controls::unipolar_from_midi;

use super::{
    device::{
        apc20::{
            AkaiApc20, Apc20ChannelButtonType, Apc20ChannelControlEvent, Apc20ControlEvent,
            Apc20StateChange,
        },
        launch_control_xl::{
            LaunchControlXLChannelButton, LaunchControlXLChannelControlEvent,
            LaunchControlXLControlEvent, LaunchControlXLStateChange, LedState,
            NovationLaunchControlXL,
        },
    },
    MidiChannelController,
};
use crate::channel::{
    ChannelControlMessage as ScopedChannelControlMessage, ControlMessage as ChannelControlMessage,
    KnobValue, StateChange as ChannelStateChange,
};

impl MidiChannelController for AkaiApc20 {
    fn interpret(&self, event: &tunnels::midi::Event) -> Option<crate::channel::ControlMessage> {
        use Apc20ChannelButtonType::*;
        use Apc20ChannelControlEvent::*;
        use Apc20ControlEvent::*;
        Some(match self.parse(event)? {
            Channel { channel, event } => match event {
                Fader(val) => ChannelControlMessage::Control {
                    channel_id: Some(channel as usize + self.channel_offset),
                    msg: ScopedChannelControlMessage::Level(unipolar_from_midi(val)),
                },
                Button(TrackSelect) => {
                    ChannelControlMessage::SelectChannel(channel as usize + self.channel_offset)
                }
            },
        })
    }

    fn emit_channel_control(
        &self,
        msg: &ChannelStateChange,
        output: &mut tunnels::midi::Output<super::Device>,
    ) {
        if let ChannelStateChange::SelectChannel(channel) = msg {
            let midi_channel = channel.inner() as isize - self.channel_offset as isize;
            let midi_channel = (midi_channel >= 0 && midi_channel < Self::CHANNEL_COUNT as isize)
                .then_some(midi_channel as u8);
            self.emit(
                Apc20StateChange::ChannelButtonRadio {
                    channel: midi_channel,
                    button: Apc20ChannelButtonType::TrackSelect,
                },
                output,
            );
        }
    }
}

impl MidiChannelController for NovationLaunchControlXL {
    fn interpret(&self, event: &tunnels::midi::Event) -> Option<crate::channel::ControlMessage> {
        use LaunchControlXLChannelButton::*;
        use LaunchControlXLChannelControlEvent::*;
        use LaunchControlXLControlEvent::*;
        Some(match self.parse(event)? {
            Channel { channel, event } => match event {
                Fader(val) => ChannelControlMessage::Control {
                    channel_id: Some(channel as usize + self.channel_offset),
                    msg: ScopedChannelControlMessage::Level(unipolar_from_midi(val)),
                },
                Knob { row, val } => ChannelControlMessage::Control {
                    channel_id: Some(channel as usize + self.channel_offset),
                    msg: ScopedChannelControlMessage::Knob {
                        index: row, // TODO: these are numbered top to bottom, do we want bottom to top?
                        value: KnobValue::Unipolar(unipolar_from_midi(val)),
                    },
                },
                Button(TrackFocus) => {
                    ChannelControlMessage::SelectChannel(channel as usize + self.channel_offset)
                }
                Button(_) => {
                    return None;
                }
            },
            SideButton(_) => {
                return None;
            }
        })
    }

    fn emit_channel_control(
        &self,
        msg: &ChannelStateChange,
        output: &mut tunnels::midi::Output<super::Device>,
    ) {
        if let ChannelStateChange::SelectChannel(channel) = msg {
            let midi_channel = self.midi_channel_for_control_channel(*channel);
            self.emit(
                LaunchControlXLStateChange::ChannelButtonRadio {
                    channel: midi_channel,
                    button: LaunchControlXLChannelButton::TrackFocus,
                    state: LedState::YELLOW,
                },
                output,
            );
        }
    }
}
