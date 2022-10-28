use std::{error::Error, time::Duration};

use crate::{
    comet::Comet, fixture::ControlMessage, lumasphere::Lumasphere, osc::OscController, Config,
};
use simple_error::bail;

pub struct Show {
    osc_controller: OscController,
    comet: Option<Comet>,
    lumasphere: Option<Lumasphere>,
}

impl Show {
    pub fn new(cfg: &Config) -> Result<Self, Box<dyn Error>> {
        let mut comet = None;
        let mut lumasphere = None;

        let mut osc_controller =
            OscController::new(cfg.receive_port, &cfg.send_host, cfg.send_port)?;

        match cfg.fixture.as_str() {
            "comet" => {
                comet = Some(Comet::new(cfg.dmx_addr));
                osc_controller.map_comet_controls();
                println!("Controlling the Comet.");
            }
            "lumasphere" => {
                lumasphere = Some(Lumasphere::new(cfg.dmx_addr));
                osc_controller.map_lumasphere_controls();
                println!("Controlling the Lumasphere.");
            }
            unknown => {
                bail!("Unknown fixture type \"{}\".", unknown);
            }
        };

        Ok(Self {
            comet,
            lumasphere,
            osc_controller,
        })
    }

    fn control(&mut self, timeout: Duration) -> Result<(), Box<dyn Error>> {
        let msg = match self.osc_controller.recv(timeout)? {
            Some(m) => m,
            None => {
                return Ok(());
            }
        };
        match msg {
            ControlMessage::Comet(c) => {
                self.comet
                    .as_mut()
                    .map(|comet| comet.control(c, &mut self.osc_controller));
            }
            ControlMessage::Lumasphere(c) => {
                self.lumasphere
                    .as_mut()
                    .map(|lumasphere| lumasphere.control(c, &mut self.osc_controller));
            }
        }
        Ok(())
    }
}
