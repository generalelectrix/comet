"""...WHO COMMANDS THE COMMANDER???"""
import argparse
import logging as log
from show_loop import run_show, ControlError
from comet import (
    Comet,
    setup_controls as setup_comet_controls,
    control_map as comet_control_map)
from venus import (
    Venus,
    setup_controls as setup_venus_controls,
    control_map as venus_control_map)
from lumasphere import Lumasphere, build_lumasphere_controls
from osc import OSCController
import pyenttec as dmx
from multiprocess import Queue
from queue import Empty
import threading
import socket
import yaml


def main(config_file):

    osc_controller = OSCController(config, control_queue)

    # which italian hot rod you like?
    fixture_choice = config['fixture']
    if fixture_choice == 'comet':
        fixture = Comet(int(config['dmx_addr']))
        setup_controls = setup_comet_controls
        control_map = comet_control_map
        log.info("Controlling a Comet.")
    elif fixture_choice == 'venus':
        fixture = Venus(int(config['dmx_addr']))
        setup_controls = setup_venus_controls
        control_map = venus_control_map
        log.info("Controlling the Venus.")
    elif fixture_choice == 'lumasphere':
        fixture = Lumasphere(int(config['dmx_addr']))
        control_map, setup_controls = build_lumasphere_controls()
        log.info("Controlling the Lumasphere.")
    else:
        log.error("Unknown fixture type: {}".format(fixture_choice))
        return

    setup_controls(osc_controller)

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

    log.info("Starting OSC server.")
    osc_thread = threading.Thread(target=osc_controller.receiver.serve_forever)
    osc_thread.start()

    def render_action(frame_number, frame_time, fixture):
        fixture.render(dmx_port.dmx_frame)
        dmx_port.render()

    try:
        run_show(
            render_action=render_action,
            control_action=process_control_event,
            update_action=lambda timestep: fixture.update(timestep),
            retrieve_show_state=lambda: fixture,
            quit_check=lambda: False,
            update_interval=int(config['update_interval']),
            report_framerate=config['debug']
        )
    finally:
        log.info("Closing OSCServer.")
        osc_controller.receiver.server_close()
        log.info("Waiting for server thread to finish.")
        osc_thread.join()
        log.info("Done.")


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--cfg', default='config.yaml')

    args = parser.parse_args()

    # fire it up!
    main(args.cfg)
