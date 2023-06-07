use anyhow::{anyhow, bail};

use crate::fixture::faderboard::{Faderboard, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::{get_unipolar, ControlMap, HandleStateChange, MapControls};

const GROUP: &str = "Faderboard";

impl MapControls for Faderboard {
    fn group(&self) -> &'static str {
        GROUP
    }
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::Faderboard;
        map.add(GROUP, "Fader", |msg| {
            let index = msg
                .addr_payload()
                .split('/')
                .take(2)
                .map(str::parse::<usize>)
                .next()
                .ok_or_else(|| anyhow!("faderboard index missing"))??;
            if index == 0 {
                bail!("Faderboard index is 0.");
            }
            let val = get_unipolar(msg)?;
            Ok(Some(Faderboard((index - 1, val))))
        })
    }
}

impl HandleStateChange<StateChange> for Faderboard {}
