from comet import Comet
import logging as log

class Patch(dict):

    def __init__(self):
        super(Patch, self).__init__()
        self.n_patched = 0

    def patch(self, dmx_addr):
        self[self.n_patched] = Comet(dmx_addr)
        self.n_patched += 1

    def unpatch(self, ID):
        try:
            del self[ID]
        except KeyError:
            log.info("Tried to unpatch nonexistant comet with ID {}".format(ID))