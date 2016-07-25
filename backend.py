from comet import Comet
import controls

from Queue import Empty

import time

def run_backend(control_queue, command_queue, dmx_port, comet_addr, debug_queue=None, debug=False, framerate=30.0):

    comet = Comet(comet_addr)

    last_render = time.time()

    render_period = 1.0 / framerate

    while True:
        # check for quit command
        try:
            command = command_queue.get(block=False)
            if command == 'quit':
                return
        except Empty:
            pass

        time_until_render = 0
        while True:
            # check if we need to render now
            time_until_render = last_render + render_period - time.time()
            if time_until_render < 0.0:
                break
            # we have some time left; wait for a UI event
            try:
                (control, value) = control_queue.get(timeout=0.9*time_until_render)
            except Empty:
                continue

            # if we got a UI event, process it
            controls.control_map[control](comet, value)
        # time to render
        comet.render(dmx_port.dmx_frame)
        dmx_port.render()
        now = time.time()
        framerate = 1.0 / (now - last_render)
        last_render = now
        if debug:
            debug_queue.put(dmx_port.dmx_frame[comet_addr:comet_addr+5])




