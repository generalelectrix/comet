//! Control for a generic strobe function.

use crate::fixture::control::{Bool, Unipolar};

#[derive(Debug)]
pub struct Strobe {
    on: Bool<()>,
    rate: Unipolar<()>,
}

impl Strobe {
    pub fn new(name: &str) -> Self {
        Self {
            on: Bool::new(format!("{name}On"), ()),
            rate: Unipolar::new(format!("{name}Rate"), ()),
        }
    }
}
