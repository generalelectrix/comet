use crate::channel::{ChannelControlMessage, ChannelStateChange, Channels};
use crate::channel::{ControlMessage, StateChange};

use crate::fixture::ControlMessagePayload;
use crate::osc::HandleOscStateChange;
use crate::osc::{GroupControlMap, MapControls, RadioButton};

use super::fader_array::FaderArray;
use super::label_array::LabelArray;

const N_CHANNELS: usize = 8;

pub(crate) const GROUP: &str = "Show";

impl Channels {
    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        CHANNEL_SELECT.map(map, |msg| ControlMessage::SelectChannel(msg));
        CHANNEL_FADERS.map(map, |channel_id, level| {
            Ok(ControlMessage::Control {
                channel_id: Some(channel_id),
                msg: ChannelControlMessage::Level(level),
            })
        });
    }
}

const CHANNEL_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Channel",
    n: N_CHANNELS,
    x_primary_coordinate: false,
};

const CHANNEL_LABELS: LabelArray = LabelArray {
    group: GROUP,
    control: "ChannelLabel",
    n: N_CHANNELS,
    empty_label: "",
};

const CHANNEL_FADERS: FaderArray = FaderArray {
    group: GROUP,
    control: "ChannelLevel",
};

impl HandleOscStateChange<StateChange> for Channels {
    fn emit_osc_state_change<S>(sc: StateChange, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        match sc {
            StateChange::SelectChannel(channel_id) => CHANNEL_SELECT.set(channel_id.into(), send),
            StateChange::ChannelLabels(labels) => CHANNEL_LABELS.set(labels.into_iter(), send),
            StateChange::State { channel_id, msg } => match msg {
                ChannelStateChange::Level(l) => CHANNEL_FADERS.set(channel_id.into(), l, send),
            },
        }
    }
}
