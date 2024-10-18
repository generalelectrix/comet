use crate::fixture::faderboard::{ControlMessage, Faderboard, StateChange};

use crate::fixture::PatchFixture;
use crate::osc::fader_array::FaderArray;
use crate::osc::{GroupControlMap, HandleOscStateChange};

const GROUP: &str = Faderboard::NAME.0;

const CONTROLS: FaderArray = FaderArray {
    group: GROUP,
    control: "Fader",
};

impl Faderboard {
    fn group(&self) -> &'static str {
        GROUP
    }

    pub fn map_controls(map: &mut GroupControlMap<ControlMessage>) {
        CONTROLS.map(map, |index, val| Ok((index, val)))
    }
}

impl HandleOscStateChange<StateChange> for Faderboard {}
