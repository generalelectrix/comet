import OSC
import logging

class OSCController(object):
    """Class to manage oversight of an external OSC control surface."""
    def __init__(self, config, control_queue):
        self.receiver = OSC.OSCServer((config['receive_host'], config['receive_port']))
        self.receiver.addMsgHandler('default', self.handle_osc_message)

        self.sender = OSC.OSCClient()
        self.sender.connect((config['send_host'], config['send_port']))
        self.control_groups = {}

        self.control_queue = control_queue

    def create_control_group(self, name):
        if name not in self.control_groups:
            self.control_groups[name] = {}

    def create_simple_control(self, group, name, control, preprocessor=None):
        """Create a pure osc listener, with no talkback."""
        if preprocessor is None:
            def callback(_, payload):
                self.send_control(control, payload)
        else:
            def callback(_, payload):
                processed = preprocessor(payload)
                self.send_control(control, processed)

        self.control_groups[group][name] = callback

    def create_radio_button_control(self, group, name, shape, control):
        """Create a radio button array control.

        This has been special-cased for present purposes.
        """
        def callback(addr, payload):
            elements = addr.split('/')
            group_name = elements[1]
            control_name = elements[2]
            base_addr = '/' + group_name + '/' + control_name + '/{}/{}'
            x = int(elements[3])
            y = int(elements[4])
            for x_but in xrange(shape[0]):
                for y_but in xrange(shape[1]):
                    this_addr = base_addr.format(x_but+1, y_but+1)
                    if x_but+1 == x and y_but+1 == y:
                        self.send_button_on(this_addr)
                    else:
                        self.send_button_off(this_addr)
            self.send_control(control, x-1)
        self.control_groups[group][name] = callback


    def handle_osc_message(self, addr, type_tags, payload, source_addr):
        elements = addr.split('/')
        if len(elements) < 3:
            return
        group_name = elements[1]
        control_name = elements[2]
        try:
            group = self.control_groups[group_name]
        except KeyError:
            logging.log("Unknown control group: {}".format(group_name))
            return
        try:
            control = group[control_name]
        except KeyError:
            logging.log("Unknown control {} in group {}"
                        .format(control_name, group_name))
        control(addr, payload[0])

    def send_control(self, control, value):
        self.control_queue.put((control, value))

    def send_button_on(self, addr):
        msg = OSC.OSCMessage()
        msg.setAddress(addr)
        msg.append(1.0)
        self.sender.send(msg)

    def send_button_off(self, addr):
        msg = OSC.OSCMessage()
        msg.setAddress(addr)
        msg.append(0.0)
        self.sender.send(msg)