use rosc::OscMessage;
use simple_error::bail;

use super::{get_unipolar, ControlMap};
use crate::faderboard::StateChange;
use crate::fixture::ControlMessage::{self as ShowControlMessage, Faderboard};

const GROUP: &str = "Faderboard";

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
    map.add(GROUP, "Fader", |msg| {
        let index = msg
            .addr
            .split("/")
            .skip(3)
            .take(2)
            .map(str::parse::<usize>)
            .next()
            .ok_or_else(|| "faderboard index missing".to_string())??;
        if index == 0 {
            bail!("Faderboard index is 0.");
        }
        let val = get_unipolar(msg)?;
        Ok(Some(Faderboard((index - 1, val))))
    })
}

pub fn handle_state_change<S>(_sc: StateChange, _send: &mut S)
where
    S: FnMut(OscMessage),
{
    // No controls with talkback.
}
