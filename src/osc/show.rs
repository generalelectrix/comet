use crate::show::ShowUIState;
use crate::show::{ControlMessage, StateChange};

use crate::fixture::ControlMessagePayload;
use crate::osc::HandleOscStateChange;
use crate::osc::{ControlMap, MapControls, RadioButton};

use super::label_array::LabelArray;

const N_ANIM_GROUP: usize = 8;

const GROUP: &str = "Show";

impl MapControls for ShowUIState {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
        ANIMATION_GROUP_SELECT.map(map, |msg| {
            ControlMessagePayload::Show(ControlMessage::SelectGroup(msg))
        });
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        Default::default()
    }
}

const ANIMATION_GROUP_SELECT: RadioButton = RadioButton {
    group: GROUP,
    control: "Group",
    n: N_ANIM_GROUP,
    x_primary_coordinate: false,
};

const ANIMATION_GROUP_LABELS: LabelArray = LabelArray {
    group: GROUP,
    control: "GroupLabel",
    n: N_ANIM_GROUP,
    empty_label: "",
};

impl HandleOscStateChange<StateChange> for ShowUIState {
    fn emit_osc_state_change<S>(sc: StateChange, send: &S)
    where
        S: crate::osc::EmitOscMessage + ?Sized,
    {
        match sc {
            StateChange::SelectGroup(msg) => ANIMATION_GROUP_SELECT.set(msg.0, send),
            StateChange::GroupLabels(labels) => {
                ANIMATION_GROUP_LABELS.set(labels.into_iter(), send)
            }
        }
    }
}
