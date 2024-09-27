use crate::show::ShowUIState;
use crate::show::{ControlMessage, StateChange};

use crate::fixture::ControlMessagePayload;
use crate::osc::HandleOscStateChange;
use crate::osc::{ControlMap, MapControls, RadioButton};

use super::label_array::LabelArray;

const N_CHANNELS: usize = 8;

const GROUP: &str = "Show";

impl MapControls for ShowUIState {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        CHANNEL_SELECT.map(map, |msg| {
            ControlMessagePayload::Show(ControlMessage::SelectChannel(msg))
        });
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        Default::default()
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

impl HandleOscStateChange<StateChange> for ShowUIState {
    fn emit_osc_state_change<S>(sc: StateChange, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        match sc {
            StateChange::SelectChannel(msg) => CHANNEL_SELECT.set(msg.0, send),
            StateChange::ChannelLabels(labels) => CHANNEL_LABELS.set(labels.into_iter(), send),
        }
    }
}
