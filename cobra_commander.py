"""...WHO COMMANDS THE COMMANDER???"""

from logging import log
from backend import DmxRenderServer
from comet_controls import setup_controls
from multiprocessing import Process, Queue
from osc import OSCController
import pyenttec as dmx
from Queue import Empty
import time
import threading
import socket
import yaml

def main():

    try:
        enttec = dmx.select_port()
    except dmx.EnttecPortOpenError as err:
        log.error(err)
        quit()

    control_queue = Queue()
    command_queue = Queue()

    # initialize control streams
    with open('config.yaml') as config_file:
        config = yaml.safe_load(config_file)

    config["receive host"] = socket.gethostbyname(socket.gethostname())
    log.info("Using local IP address {}".format(config["receive host"]))
    osc_controller = OSCController(config, control_queue)
    setup_controls(osc_controller)

    debug = config["debug"]
    if debug:
        debug_queue = Queue()
    else:
        debug_queue = None

    backend = Process(
        target=run_backend,
        args=(control_queue, command_queue, enttec, config['dmx_addr']-1, debug_queue))

    backend.start()

    log.info("\nStarting OSCServer.")
    osc_thread = threading.Thread(target=osc_controller.receiver.serve_forever)
    osc_thread.start()

    try:
        while True:
            if debug:
                try:
                    log.info(debug_queue.get(block=False))
                except Empty:
                    time.sleep(0.1)
            else:
                user_input = raw_input('Enter q to quit.')
                if user_input == 'q':
                    break


    finally:
        command_queue.put('quit')
        log.info("\nClosing OSCServer.")
        osc_controller.receiver.close()
        log.info("Waiting for Server-thread to finish")
        osc_thread.join() ##!!!
        log.info("Done")

if __name__ == '__main__':
    # fire it up!
    main()


def run(config, update_interval=20, n_frames=None, control_timeout=0.001):
    """Run the show loop.

    Args:
        update_interval (int): number of milliseconds between beam state updates
        n_frames (None or int): if None, run forever.  if finite number, only
            run for this many state updates.
    """

    report_framerate = config["report_framerate"]

    update_number = 0

    # start up the render server
    render_server = DmxRenderServer(report=report_framerate)

    log.info("Starting render server...")
    render_server.start()
    log.info("Render server started.")

    time_millis = lambda: int(time.time()*1000)

    last_update = time_millis()

    last_rendered_frame = -1

    try:
        while n_frames is None or update_number < n_frames:
            # process a control event if one is pending
            try:
                self.midi_in.receive(timeout=control_timeout)
            except Empty:
                # fine if we didn't get a control event
                pass

            # compute updates until we're current
            now = time_millis()
            time_since_last_update = now - last_update

            while time_since_last_update > update_interval:
                # update the state of the beams
                for layer in self.mixer.layers:
                    layer.beam.update_state(update_interval)

                last_update += update_interval
                now = time_millis()
                time_since_last_update = now - last_update
                update_number += 1


            # pass the mixer the render process is ready to draw another frame
            # and it hasn't drawn this frame yet
            if update_number > last_rendered_frame:
                rendered = render_server.pass_frame_if_ready(
                    update_number, last_update, self.mixer)
                if rendered:
                    last_rendered_frame = update_number

    finally:
        render_server.stop()
        log.info("Shut down render server.")




