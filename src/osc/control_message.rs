use lazy_static::lazy_static;
use regex::Regex;
use rosc::{OscMessage, OscType};

use crate::fixture::Group;

use super::OscError;

/// Wrapper type for OSC messages that provides a simplification for our domain.
/// This includes pre-processing of the address to identify the breaks, as well
/// as parsing of the group ID (or filling in a placeholder).
#[derive(Debug)]
pub struct OscControlMessage {
    /// The raw/full OSC address.
    addr: String,
    /// Single OSC payload extracted from the incoming message.
    pub arg: OscType,
    /// The byte index in the addr string where the control key starts.
    key_start: usize,
    /// The byte index in the addr string of the first character after the
    /// control key. For addrs with no payload following the control key,
    /// this may be equal to the length of the address and thus we must be
    /// careful not to accidentally try to slice past the end of the address.
    key_end: usize,
    /// The group ID, if present.
    pub group: Group,
}

impl OscControlMessage {
    pub fn new(msg: OscMessage) -> Result<Self, OscError> {
        let wrap_err = |m| OscError {
            addr: msg.addr.clone(),
            msg: m,
        };

        let (key_start, key_end, group) = parse_address(&msg.addr).map_err(wrap_err)?;
        let arg = get_single_arg(msg.args).map_err(wrap_err)?;

        Ok(Self {
            addr: msg.addr,
            arg,
            key_start,
            key_end,
            group,
        })
    }

    /// Return the control key portion of the address.
    pub fn key(&self) -> &str {
        &self.addr[self.key_start..self.key_end]
    }

    /// Return the portion of the address following the control key.
    pub fn addr_payload(&self) -> &str {
        if self.key_end == self.addr.len() {
            return "";
        }
        &self.addr[self.key_end..]
    }

    /// Generate an OscError.
    pub fn err<M: Into<String>>(&self, msg: M) -> OscError {
        OscError {
            addr: self.addr.to_string(),
            msg: msg.into(),
        }
    }
}

fn parse_address(addr: &str) -> Result<(usize, usize, Group), String> {
    lazy_static! {
        static ref WITH_GROUP: Regex = Regex::new(r"^/:([^/]+)(/[^/]+/[^/]+)").unwrap();
        static ref WITHOUT_GROUP: Regex = Regex::new(r"^(/[^:/][^/]*/[^/]+)").unwrap();
    }

    if let Some(caps) = WITH_GROUP.captures(addr) {
        let key_match = caps.get(2).unwrap();
        return Ok((key_match.start(), key_match.end(), Group::new(&caps[1])));
    }
    if let Some(caps) = WITHOUT_GROUP.captures(addr) {
        let key_match = caps.get(1).unwrap();
        return Ok((key_match.start(), key_match.end(), Group::none()));
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
    use super::*;
    use rosc::OscType;
    #[test]
    fn test_get_control_key() {
        assert_eq!("/foo/bar", get_control_key("/:hello/foo/bar/baz").unwrap());
        assert_eq!("/foo/bar", get_control_key("/foo/bar/baz").unwrap());
        assert_eq!("/foo/bar", get_control_key("/foo/bar").unwrap());
        let bad = vec!["", "foo", "foo/bar", "/bar", "/", "/:foo/bar"];
        for b in bad.iter() {
            assert!(get_control_key(b).is_err());
        }
    }

    fn get_control_key(addr: &str) -> Result<String, OscError> {
        let msg = OscControlMessage::new(OscMessage {
            addr: addr.to_string(),
            args: vec![OscType::Nil],
        })?;
        Ok(msg.key().to_string())
    }
}
