use anyhow::{anyhow, bail, Context};

use crate::fixture::faderboard::{Faderboard, StateChange};
use crate::fixture::FixtureControlMessage;
use crate::osc::{get_unipolar, ControlMap, HandleStateChange, MapControls};

const GROUP: &str = "Faderboard";

impl MapControls for Faderboard {
    fn map_controls(&self, map: &mut ControlMap<FixtureControlMessage>) {
        use FixtureControlMessage::Faderboard;
        map.add(GROUP, "Fader", |msg| {
            let index = msg
                .addr_payload()
                .split('/')
                .skip(1)
                .take(1)
                .next()
                .ok_or_else(|| anyhow!("faderboard index missing"))?
                .parse::<usize>()
                .with_context(|| format!("handling message {msg:?}"))?;
            if index == 0 {
                bail!("Faderboard index is 0.");
            }
            let val = get_unipolar(msg)?;
            Ok(Some(Faderboard((index - 1, val))))
        })
    }
}

impl HandleStateChange<StateChange> for Faderboard {}
