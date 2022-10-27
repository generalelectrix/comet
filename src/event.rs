use crossbeam_channel::{unbounded, Receiver, RecvError, Sender, TryRecvError};
use std::{
    error::Error,
    thread::{spawn, JoinHandle},
    time::{Duration, Instant},
};

use log::{error, info};
use simple_error::{bail, simple_error};

pub fn run<S: Show + 'static>(
    mut show: S,
    control_timeout: Duration,
    update_interval: Duration,
    n_frames: Option<usize>,
    report_framerate: bool,
) -> Result<(), Box<dyn Error>> {
    let mut update_number = 0;

    info!("Starting render server...");
    let render_server: RenderServer<S::F> = RenderServer::new(report_framerate)?;
    info!("Render server started.");

    let mut last_update = Instant::now();
    let mut last_rendered_frame = 0;
    loop {
        if let Some(frame_run_count) = n_frames {
            if frame_run_count <= update_number {
                break;
            }
        }
        if show.quit() {
            break;
        }

        // Process a control event if one is pending.
        if let Err(err) = show.control(control_timeout) {
            error!("A control error occurred: {}.", err);
        }

        // Compute updates until we're current.
        let mut now = Instant::now();
        let mut time_since_last_update = now - last_update;
        while time_since_last_update > update_interval {
            // Update the state of the show.
            show.update(update_interval);

            last_update += update_interval;
            now = Instant::now();
            time_since_last_update = now - last_update;
            update_number += 1;
        }

        // Pass the show state to the render process if it is ready to
        // draw another frame and it hasn't drawn this frame yet.
        if update_number > last_rendered_frame {
            match render_server.pass_frame_if_ready(update_number, last_update, show.state()) {
                Err(e) => {
                    error!("Render server returned an error: {}. Shutting down.", e);
                    break;
                }
                Ok(rendered) => {
                    if rendered {
                        last_rendered_frame = update_number;
                    }
                }
            }
        }
    }
    Ok(())
}

pub trait Show {
    type F: Frame;
    fn quit(&self) -> bool;
    fn control(&mut self, timeout: Duration) -> Result<(), Box<dyn Error>>;
    fn update(&mut self, delta_t: Duration);
    fn state(&self) -> Self::F;
}

enum RenderServerCommand<F> {
    Frame(RenderFrame<F>),
    Quit,
}

struct RenderFrame<F> {
    frame: F,
    update_number: usize,
    update_time: Instant,
}

enum RenderServerResponse {
    Running,
    FrameReq,
    FatalError(Box<dyn Error + Send>),
}

struct RenderServer<F> {
    command: Sender<RenderServerCommand<F>>,
    response: Receiver<RenderServerResponse>,
    stop_render_thread: Option<Box<dyn FnOnce() -> ()>>,
    report: bool,
}

impl<F: Frame + 'static> RenderServer<F> {
    fn new(report: bool) -> Result<Self, Box<dyn Error>> {
        use RenderServerResponse::*;
        let (cmd_send, cmd_recv) = unbounded::<RenderServerCommand<F>>();
        let (resp_send, resp_recv) = unbounded::<RenderServerResponse>();
        let render_thread = spawn(move || {
            run_render_server(cmd_recv, resp_send, report);
        });

        match resp_recv.recv() {
            Ok(r) => match r {
                Running => (),
                FrameReq => {
                    // Unclear how this happened.
                    cmd_send.send(RenderServerCommand::Quit);
                    render_thread.join();
                    bail!("render server asked for a frame before reporting Running");
                }
                FatalError(err) => {
                    render_thread.join();
                    bail!("render server reported a fatal error: {}", err);
                }
            },
            Err(_) => {
                render_thread.join();
                bail!("render server hung up unexpectedly");
            }
        }

        Ok(Self {
            command: cmd_send.clone(),
            response: resp_recv,
            stop_render_thread: Some(Box::new(move || {
                cmd_send.send(RenderServerCommand::Quit);
                render_thread.join();
            })),
            report,
        })
    }

    /// Pass the render server a frame if it is ready to draw one.
    /// Return a boolean indicating if a frame was drawn or not.
    pub fn pass_frame_if_ready(
        &self,
        update_number: usize,
        update_time: Instant,
        frame: F,
    ) -> Result<bool, Box<dyn Error>> {
        match self.response.try_recv() {
            Err(e) => match e {
                TryRecvError::Empty => Ok(false),
                TryRecvError::Disconnected => {
                    bail!("render server response channel disconnected");
                }
            },
            Ok(m) => match m {
                RenderServerResponse::Running => {
                    bail!("render server sent running signal again");
                }
                RenderServerResponse::FatalError(e) => Err(e),
                RenderServerResponse::FrameReq => {
                    if let Err(_) = self.command.send(RenderServerCommand::Frame(RenderFrame {
                        frame,
                        update_number,
                        update_time,
                    })) {
                        bail!("render server control channel disconnected");
                    }
                    Ok(true)
                }
            },
        }
    }
}

impl<F> Drop for RenderServer<F> {
    fn drop(&mut self) {
        self.stop_render_thread.take().map(|stop| stop());
        info!("Shut down render server.");
    }
}

pub trait Frame: Send {
    fn render<T>(&self, renderer: &mut T) -> Result<(), Box<dyn Error>>;
}

/// Run the frame drawing service.
fn run_render_server<F: Frame>(
    recv: Receiver<RenderServerCommand<F>>,
    resp: Sender<RenderServerResponse>,
    report: bool,
) {
    let mut frame_number = 0;
    if let Err(_) = resp.send(RenderServerResponse::Running) {
        error!("Render service response channel hung up unexpectedly.");
        return;
    }
    let mut log_time = Instant::now();
    loop {
        if let Err(_) = resp.send(RenderServerResponse::FrameReq) {
            error!("Render service response channel hung up unexpectedly.");
            return;
        }
    }
}
