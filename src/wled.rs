//! Use a thread to perform asynchronous communication with a WLED instance.
use anyhow::Result;
use log::{error, info};
use std::sync::mpsc::{channel, Sender};

use anyhow::Context;
use reqwest::Url;
use wled_json_api_library::{structures::state::State, wled::Wled};

use crate::control::ControlMessage;

pub struct WledController {
    send: Sender<WledControlMessage>,
}

impl WledController {
    /// Run a thread to handle sending WLED control messages.
    pub fn run(addr: &str, _send: Sender<ControlMessage>) -> Result<Self> {
        let mut wled = Wled::try_from_url(&Url::try_from(addr).context("parsing WLED URL")?)
            .context("creating WLED instance")?;
        let (send_state, recv_state) = channel();
        std::thread::spawn(move || {
            for msg in recv_state {
                match msg {
                    WledControlMessage::SetState(state) => {
                        wled.state = Some(state);
                        info!("Sending state.");
                        if let Err(err) = wled.flush_state() {
                            error!("failed to send WLED state update: {err}");
                            continue;
                        }
                    }
                    WledControlMessage::GetEffectMetadata => {
                        // TODO
                        error!("fxdata not implemented");
                        continue;
                    }
                }
            }
            info!("WLED handler thread shutting down.");
        });
        Ok(Self { send: send_state })
    }
}

pub enum WledControlMessage {
    SetState(State),
    GetEffectMetadata,
}

pub enum WledResponse {}

pub trait EmitWledControlMessage {
    fn emit_wled(&self, msg: WledControlMessage);
}

impl EmitWledControlMessage for WledController {
    fn emit_wled(&self, msg: WledControlMessage) {
        if self.send.send(msg).is_err() {
            error!("Error emitting WLED control message: message processor channel has hung up.");
        }
    }
}
