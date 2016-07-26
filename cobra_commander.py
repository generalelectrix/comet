"""...WHO COMMANDS THE COMMANDER???"""

from logging import log
from show_loop import run_show, ControlError
from comet import Comet
from comet_controls import setup_controls, control_map
from osc import OSCController
import pyenttec as dmx
from multiprocessing import Queue
from Queue import Empty
import threading
import socket
import yaml

def main():

    try:
        dmx_port = dmx.select_port()
    except dmx.EnttecPortOpenError as err:
        log.error(err)
        quit()

    control_queue = Queue()

    # initialize control streams
    with open('config.yaml') as config_file:
        config = yaml.safe_load(config_file)

    config["receive host"] = socket.gethostbyname(socket.gethostname())
    log.info("Using local IP address {}".format(config["receive host"]))
    osc_controller = OSCController(config, control_queue)
    setup_controls(osc_controller)

    fixture = Comet(int(config['dmx_addr']))

    def process_control_event(timeout):
        """Drain the control queue and apply the action."""
        try:
            (control, value) = control_queue.get(timeout=timeout)
        except Empty:
            pass
        else:
            # if we got a UI event, process it
            try:
                control_map[control](fixture, value)
            except KeyError:
                raise ControlError("Unknown control: '{}'".format(control))

    log.info("\nStarting OSCServer.")
    osc_thread = threading.Thread(target=osc_controller.receiver.serve_forever)
    osc_thread.start()

    def render_action(frame_number, frame_time, fixture):
        fixture.render(dmx_port.dmx_frame)
        dmx_port.render()

    try:
        run_show(
            render_action=render_action,
            control_action=process_control_event,
            update_action=lambda _: None,
            retrieve_show_state=lambda: fixture,
            quit_check=lambda: False,
            update_interval=int(config['update_interval']),
            report_framerate=config['debug']
            )

    finally:
        log.info("\nClosing OSCServer.")
        osc_controller.receiver.close()
        log.info("Waiting for Server-thread to finish")
        osc_thread.join() ##!!!
        log.info("Done")

if __name__ == '__main__':
    # fire it up!
    main()


