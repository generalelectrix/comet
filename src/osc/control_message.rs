use std::ops::Range;

use lazy_static::lazy_static;
use regex::Regex;
use rosc::{OscMessage, OscType};

use super::{OscClientId, OscError};

/// Wrapper type for OSC messages that provides a simplification for our domain.
/// This includes pre-processing of the address to identify the breaks, as well
/// as parsing of the group ID.
#[derive(Debug)]
pub struct OscControlMessage {
    /// The ID of the client that originated this message.
    pub client_id: OscClientId,
    /// The raw/full OSC address.
    addr: String,
    /// Single OSC payload extracted from the incoming message.
    pub arg: OscType,
    addr_index: AddressIndex,
}

#[derive(Debug)]
struct AddressIndex {
    /// The byte index in the addr string where the control key starts,
    /// including the leading slash.
    key_start: usize,
    /// The byte index in the addr string of the first character of the control
    /// portion of the address, including the leading slash.
    control_start: usize,
    /// The byte index in the addr string of the first character after the
    /// control key. For addrs with no payload following the control key,
    /// this may be equal to the length of the address and thus we must be
    /// careful not to accidentally try to slice past the end of the address.
    key_end: usize,
    /// The byte range of the group ID, if present.
    group: Option<Range<usize>>,
}

impl OscControlMessage {
    pub fn new(msg: OscMessage, client_id: OscClientId) -> Result<Self, OscError> {
        let wrap_err = |m| OscError {
            addr: msg.addr.clone(),
            msg: m,
        };

        let addr_index = parse_address(&msg.addr).map_err(wrap_err)?;
        let arg = get_single_arg(msg.args).map_err(wrap_err)?;

        Ok(Self {
            client_id,
            addr: msg.addr,
            arg,
            addr_index,
        })
    }

    /// Return the first half of the control key, excluding the leading slash.
    pub fn entity_type(&self) -> &str {
        &self.addr[self.addr_index.key_start + 1..self.addr_index.control_start]
    }

    /// Return the control portion of the address.
    pub fn control(&self) -> &str {
        &self.addr[self.addr_index.control_start + 1..self.addr_index.key_end]
    }

    /// Return the group, if present.
    pub fn group(&self) -> Option<&str> {
        Some(&self.addr[self.addr_index.group.as_ref()?.clone()])
    }

    /// Return the portion of the address following the control key.
    /// This will include a leading / if not empty.
    pub fn addr_payload(&self) -> &str {
        if self.addr_index.key_end == self.addr.len() {
            return "";
        }
        &self.addr[self.addr_index.key_end..]
    }

    /// Generate an OscError.
    pub fn err<M: Into<String>>(&self, msg: M) -> OscError {
        OscError {
            addr: self.addr.to_string(),
            msg: msg.into(),
        }
    }
}

fn parse_address(addr: &str) -> Result<AddressIndex, String> {
    lazy_static! {
        static ref WITH_GROUP: Regex = Regex::new(r"^/:([^/]+)(/[^/]+)(/[^/]+)").unwrap();
        static ref WITHOUT_GROUP: Regex = Regex::new(r"^(/[^:/][^/]*)(/[^/]+)").unwrap();
    }

    if let Some(caps) = WITH_GROUP.captures(addr) {
        let group_match = caps.get(1).unwrap();
        let key_match = caps.get(2).unwrap();
        let control_match = caps.get(3).unwrap();
        return Ok(AddressIndex {
            key_start: key_match.start(),
            control_start: control_match.start(),
            key_end: control_match.end(),
            group: Some(group_match.start()..group_match.end()),
        });
    }
    if let Some(caps) = WITHOUT_GROUP.captures(addr) {
        let key_match = caps.get(1).unwrap();
        let control_match = caps.get(2).unwrap();
        return Ok(AddressIndex {
            key_start: key_match.start(),
            control_start: control_match.start(),
            key_end: control_match.end(),
            group: None,
        });
    }
    Err("address did not match expected patterns".to_string())
}

fn get_single_arg(mut args: Vec<OscType>) -> Result<OscType, String> {
    if args.len() > 1 {
        Err(format!("message has {} args (expected one)", args.len()))
    } else if args.is_empty() {
        Err("message has empty args list".to_string())
    } else {
        Ok(args.pop().unwrap())
    }
}

#[cfg(test)]
mod test {
    use std::{net::SocketAddr, str::FromStr};

    use super::*;
    use rosc::OscType;
    #[test]
    fn test_get_control_key() {
        assert_eq!(
            ("foo".to_string(), "bar".to_string()),
            get_control_key("/:hello/foo/bar/baz").unwrap()
        );
        assert_eq!(
            ("foo".to_string(), "bar".to_string()),
            get_control_key("/foo/bar/baz").unwrap()
        );
        assert_eq!(
            ("foo".to_string(), "bar".to_string()),
            get_control_key("/foo/bar").unwrap()
        );
        let bad = ["", "foo", "foo/bar", "/bar", "/", "/:foo/bar"];
        for b in bad.iter() {
            assert!(get_control_key(b).is_err());
        }
    }

    fn get_control_key(addr: &str) -> Result<(String, String), OscError> {
        let msg = OscControlMessage::new(
            OscMessage {
                addr: addr.to_string(),
                args: vec![OscType::Nil],
            },
            OscClientId(SocketAddr::from_str("127.0.0.1:1234").unwrap()),
        )?;
        Ok((msg.entity_type().to_string(), msg.control().to_string()))
    }
}
