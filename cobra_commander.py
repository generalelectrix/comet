"""...WHO COMMANDS THE COMMANDER???"""
from __future__ import print_function

from backend import run_backend
from controls import setup_controls
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
        print(err)
        quit()

    control_queue = Queue()
    command_queue = Queue()

    # initialize control streams
    with open('config.yaml') as config_file:
        config = yaml.safe_load(config_file)

    config["receive host"] = socket.gethostbyname(socket.gethostname())
    print("Using local IP address {}".format(config["receive host"]))
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

    print("\nStarting OSCServer.")
    osc_thread = threading.Thread(target=osc_controller.receiver.serve_forever)
    osc_thread.start()

    try:
        while True:
            if debug:
                try:
                    print(debug_queue.get(block=False))
                except Empty:
                    time.sleep(0.1)
            else:
                user_input = raw_input('Enter q to quit.')
                if user_input == 'q':
                    break


    finally:
        command_queue.put('quit')
        print("\nClosing OSCServer.")
        osc_controller.receiver.close()
        print("Waiting for Server-thread to finish")
        osc_thread.join() ##!!!
        print("Done")

if __name__ == '__main__':
    # fire it up!
    main()




