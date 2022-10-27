use std::{error::Error, time::Duration};

use crate::{comet::Comet, lumasphere::Lumasphere, Config};
use simple_error::bail;

pub struct Show {
    comet: Option<Comet>,
    lumasphere: Option<Lumasphere>,
}

impl Show {
    pub fn new(cfg: &Config) -> Result<Self, Box<dyn Error>> {
        let mut comet = None;
        let mut lumasphere = None;

        match cfg.fixture.as_str() {
            "comet" => {
                comet = Some(Comet::new(cfg.dmx_addr));
                println!("Controlling the Comet.");
            }
            "lumasphere" => {
                lumasphere = Some(Lumasphere::new(cfg.dmx_addr));
                println!("Controlling the Lumasphere.");
            }
            unknown => {
                bail!("Unknown fixture type \"{}\".", unknown);
            }
        };

        Ok(Self { comet, lumasphere })
    }

    fn control(&mut self, timeout: Duration) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}
