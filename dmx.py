import serial, sys

START_VAL   = 0x7E
END_VAL     = 0xE7

COM_BAUD    = 57600
COM_TIMEOUT = 1
COM_PORT    = 7
DMX_SIZE    = 512

LABELS = {
         'GET_WIDGET_PARAMETERS' :3,  #unused
         'SET_WIDGET_PARAMETERS' :4,  #unused
         'RX_DMX_PACKET'         :5,  #unused
         'TX_DMX_PACKET'         :6,
         'TX_RDM_PACKET_REQUEST' :7,  #unused
         'RX_DMX_ON_CHANGE'      :8,  #unused
      }

packet_start = [START_VAL,
                LABELS['TX_DMX_PACKET'],
                (DMX_SIZE + 1) & 0xFF,
                ( (DMX_SIZE + 1) >> 8) & 0xFF,
                0]
packet_start = ''.join([chr(v) for v in packet_start])
packet_end = chr(END_VAL)

class DMXConnection(object):
    def __init__(self, comport = None):
        '''
        On Windows, the only argument is the port number. On *nix, it's the path to the serial device.
        For example:
            DMXConnection(4)              # Windows
            DMXConnection('/dev/tty2')    # Linux
            DMXConnection("/dev/ttyUSB0") # Linux
        '''
        self.dmx_frame = [0] * DMX_SIZE
        try:
          self.com = serial.Serial(comport, baudrate = COM_BAUD, timeout = COM_TIMEOUT)
        except:
          com_name = 'COM%s' % (comport + 1) if type(comport) == int else comport
          print "Could not open device %s. Quitting application." % com_name
          sys.exit(0)

        print "Opened %s." % (self.com.portstr)


    def render(self):
        ''''
        Updates the DMX output from the USB DMX Pro with the values from self.dmx_frame.
        '''
        dmx_payload = (chr(v) for v in self.dmx_frame)

        self.com.write(packet_start + ''.join(dmx_payload) + packet_end)

    def close(self):
        self.com.close()