use anyhow::{anyhow, bail, Context};

use crate::fixture::faderboard::{Faderboard, StateChange};
use crate::fixture::ControlMessagePayload;
use crate::fixture::PatchFixture;
use crate::osc::{get_unipolar, ControlMap, HandleOscStateChange, MapControls};

const GROUP: &str = "Faderboard";

impl MapControls for Faderboard {
    fn map_controls(&self, map: &mut ControlMap<ControlMessagePayload>) {
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
            Ok(Some(ControlMessagePayload::fixture((index - 1, val))))
        })
    }

    fn fixture_type_aliases(&self) -> Vec<(String, crate::fixture::FixtureType)> {
        vec![(GROUP.to_string(), Self::NAME)]
    }
}

impl HandleOscStateChange<StateChange> for Faderboard {}
