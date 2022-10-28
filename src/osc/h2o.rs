use std::error::Error;
use std::str::FromStr;

use rosc::{OscMessage, OscType};
use simple_error::bail;

use super::ControlMap;
use crate::fixture::ControlMessage::{self as ShowControlMessage, H2O};
use crate::h2o::{FixedColor, StateChange};
use crate::util::bipolar_fader_with_detent;
use crate::util::unipolar_fader_with_detent;
use strum::IntoEnumIterator;

const GROUP: &str = "H2O";
const FIXED_COLOR: &str = "FixedColor";

pub fn map_controls(map: &mut ControlMap<ShowControlMessage>) {
    use StateChange::*;
    map.add(GROUP, FIXED_COLOR, parse_fixed_color);
}

fn parse_fixed_color(m: OscMessage) -> Result<Option<ShowControlMessage>, Box<dyn Error>> {
    let color_name = match m.addr.split("/").skip(3).next() {
        Some(c) => c,
        None => {
            bail!("fixed color command is missing color name: {}", m.addr);
        }
    };
    let color = FixedColor::from_str(color_name)?;
    Ok(Some(H2O(StateChange::FixedColor(color))))
}

pub fn handle_state_change<S>(sc: StateChange, send: &mut S)
where
    S: FnMut(OscMessage),
{
    match sc {
        StateChange::FixedColor(c) => {
            // Essentially a radio button implementation.
            for color_choice in FixedColor::iter() {
                send(OscMessage {
                    addr: format!("/{}/{}/{}", GROUP, FIXED_COLOR, color_choice),
                    args: vec![OscType::Float(if color_choice == c { 1.0 } else { 0.0 })],
                });
            }
        }
        _ => (),
    }
}
