from luna import configure_default_logging

from apollo_fpga import ApolloDebugger
from apollo_fpga.ecp5 import ECP5_JTAGProgrammer

import logging
import sys


if __name__ == "__main__":
    # parse arguments
    if len(sys.argv) != 2:
        print("Usage: configure_ecp5 <filename>")
        sys.exit(1)
    filename = sys.argv[1]

    # configure logging
    configure_default_logging()
    logging.getLogger().setLevel(logging.DEBUG)

    # create debugger
    debugger = ApolloDebugger()

    # read bitstream
    logging.info(f"Reading bitstream: {filename}")
    with open(filename, "rb") as f:
        bitstream = f.read()

    # load bitstream
    with debugger.jtag as jtag:
        programmer = ECP5_JTAGProgrammer(jtag)
        try:
            logging.info(f"Uploading bitstream")
            programmer.configure(bitstream)
        except Exception as e:
            logging.error(f"Error uploading bitstream: {e}")
            sys.exit(1)

    logging.info(f"Success")
