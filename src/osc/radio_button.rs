use rosc::{OscMessage, OscType};
use simple_error::bail;
use std::{error::Error, fmt::Display, str::FromStr};
use strum::IntoEnumIterator;

use super::{control_message::OscControlMessage, OscError};

/// Model a 1D button grid with radio-select behavior.
/// This implements the TouchOSC model for a button grid.
/// Special-cased to handle only 1D grids.
#[derive(Clone)]
pub struct RadioButton {
    pub group: &'static str,
    pub control: &'static str,
    pub n: usize,
    /// If true, use the 0th coordinate as the index.  If false, use the 1st coordinate.
    /// FIXME: this forces us to encode the orientation of the TouchOSC layout into
    /// the control profile.  We might want to replace the button grids with individual
    /// buttons in the future to fix this.
    pub x_primary_coordinate: bool,
}

impl RadioButton {
    /// Get a index from a collection of radio buttons, mapped to numeric addresses.
    pub fn parse(&self, v: &OscControlMessage) -> Result<usize, OscError> {
        let (x, y) = match parse_radio_button_indices(v.addr_payload()) {
            Ok(indices) => indices,
            Err(err) => {
                return Err(v.err(err));
            }
        };
        let (primary, secondary) = if self.x_primary_coordinate {
            (x, y)
        } else {
            (y, x)
        };
        if primary >= self.n {
            return Err(v.err(format!(
                "radio button primary index out of range: {}",
                primary
            )));
        }
        if secondary > 0 {
            return Err(v.err(format!(
                "radio button secondary index out of range: {}",
                secondary
            )));
        }
        Ok(primary)
    }

    /// Send OSC messages to set the current state of the button.
    pub fn set<S>(&self, n: usize, send: &mut S) -> Result<(), Box<dyn Error>>
    where
        S: FnMut(OscMessage),
    {
        if n >= self.n {
            bail!(
                "radio button index {} out of range for {}/{}",
                n,
                self.group,
                self.control
            );
        }
        for i in 0..self.n {
            let val = if i == n { 1.0 } else { 0.0 };
            let (x, y) = if self.x_primary_coordinate {
                (i + 1, 1)
            } else {
                (1, i + 1)
            };
            send(OscMessage {
                addr: format!("/{}/{}/{}/{}", self.group, self.control, x, y),
                args: vec![OscType::Float(val)],
            })
        }
        Ok(())
    }
}

/// Parse radio button indices from a TouchOSC button grid.
fn parse_radio_button_indices(addr_payload: &str) -> Result<(usize, usize), String> {
    let mut pieces_iter = addr_payload
        .split("/")
        .skip(1)
        .take(2)
        .map(str::parse::<usize>);
    let x = pieces_iter
        .next()
        .ok_or_else(|| "x radio button index missing".to_string())?
        .map_err(|err| format!("failed to parse radio button x index: {}", err))?;
    let y = pieces_iter
        .next()
        .ok_or_else(|| "y radio button index missing".to_string())?
        .map_err(|err| format!("failed to parse radio button y index: {}", err))?;
    if x == 0 {
        return Err(format!("x index is unexpectedly 0"));
    }
    if y == 0 {
        return Err(format!("y index is unexpectedly 0"));
    }
    Ok((x - 1, y - 1))
}

pub trait EnumRadioButton: FromStr + IntoEnumIterator + Display + PartialEq
where
    <Self as FromStr>::Err: std::fmt::Display,
{
    /// Parse a enum variant of the specified type from the third argument of the address.
    fn parse(m: &OscControlMessage) -> Result<Self, OscError> {
        let name = match m.addr_payload().split("/").skip(1).next() {
            Some(c) => c,
            None => {
                return Err(m.err("command is missing variant specifier"));
            }
        };
        Self::from_str(name).map_err(|err| m.err(err.to_string()))
    }

    /// Update the state of a "radio select enum".
    /// Each enum variant is mapped to a button with the name of the address as the
    /// last piece of the address.
    fn set<S>(&self, group: &str, control: &str, send: &mut S)
    where
        S: FnMut(OscMessage),
    {
        for choice in Self::iter() {
            send(OscMessage {
                addr: format!("/{}/{}/{}", group, control, choice),
                args: vec![OscType::Float(if choice == *self { 1.0 } else { 0.0 })],
            });
        }
    }
}
