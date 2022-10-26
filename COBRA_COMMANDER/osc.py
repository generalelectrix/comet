class OSCController(object):
    """Class to manage oversight of an external OSC control surface."""

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
            for x_but in range(shape[0]):
                for y_but in range(shape[1]):
                    this_addr = base_addr.format(x_but+1, y_but+1)
                    if x_but+1 == x and y_but+1 == y:
                        self.send_button_on(this_addr)
                    else:
                        self.send_button_off(this_addr)
            self.send_control(control, x-1)
        self.control_groups[group][name] = callback

    def send_button_on(self, addr):
        self.sender.send_message(addr, 1.0)

    def send_button_off(self, addr):
        self.sender.send_message(addr, 0.0)
