use rosc::OscMessage;

use super::ControlMap;
use crate::fixture::ControlMessage::{self as ShowControlMessage, FreedomFries};
use crate::freedom_fries::{FreedomFries as FreedomFriesFixture, StateChange};
use crate::util::unipolar_to_range;

const GROUP: &str = "FreedomFries";

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
    use StateChange::*;

    map.add_unipolar(GROUP, "Dimmer", |v| FreedomFries(Dimmer(v)));
    map.add_unipolar(GROUP, "Speed", |v| FreedomFries(Speed(v)));
    map.add_unipolar(GROUP, "Program", |v| {
        FreedomFries(Program(
            unipolar_to_range(0, FreedomFriesFixture::PROGRAM_COUNT as u8 - 1, v) as usize,
        ))
    });
    map.add_bool(GROUP, "ProgramCycleAll", |v| {
        FreedomFries(ProgramCycleAll(v))
    });
}

pub fn handle_state_change<S>(_sc: StateChange, _send: &mut S)
where
    S: FnMut(OscMessage),
{
    // No controls with talkback.
}
