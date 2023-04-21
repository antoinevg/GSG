#!/usr/bin/env python3
# pylint: disable=no-member
#
# This file is part of LUNA.
#
# Copyright (c) 2023 Great Scott Gadgets <info@greatscottgadgets.com>
# SPDX-License-Identifier: BSD-3-Clause

import os
import sys
import logging
import time

from enum import IntEnum

import usb1

from luna import configure_default_logging

VENDOR_ID  = 0x16d0
PRODUCT_ID = 0x0f3b

BULK_ENDPOINT_NUMBER = 1
COMMAND_ENDPOINT_NUMBER = 2

# Set the total amount of data to be used in our speed test.
TEST_DATA_SIZE = 1 * 1024 * 1024
TEST_TRANSFER_SIZE = 16 * 1024

# Size of the host-size "transfer queue" -- this is effectively the number of async transfers we'll
# have scheduled at a given time.
TRANSFER_QUEUE_DEPTH = 16

# Test commands
class TestCommand(IntEnum):
    Stop = 0x01,
    In   = 0x23,
    Out  = 0x42,

# Error messages
_messages = {
    1: "error'd out",
    2: "timed out",
    3: "was prematurely cancelled",
    4: "was stalled",
    5: "lost the device it was connected to",
    6: "sent more data than expected."
}


# - IN Speed Test ------------------------------------------------------------

def run_in_speed_test():
    """ Runs a simple IN speed test, and reports throughput. """

    total_data_exchanged = 0
    failed_out = False

    def _should_terminate():
        """ Returns true iff our test should terminate. """
        return (total_data_exchanged > TEST_DATA_SIZE) or failed_out


    def _transfer_completed(transfer: usb1.USBTransfer):
        """ Callback executed when an async transfer completes. """
        nonlocal total_data_exchanged, failed_out

        status = transfer.getStatus()

        # If the transfer completed.
        if status in (usb1.TRANSFER_COMPLETED,):

            # Count the data exchanged in this packet...
            total_data_exchanged += transfer.getActualLength()
            logging.debug(f"usb1.TRANSFER_COMPLETED: {total_data_exchanged} bytes")

            #buffer = transfer.getBuffer()
            #logging.info(f"  buffer length: {len(buffer)}")
            #logging.info(f"  {list(buffer[:8])} -> {list(buffer[-8:])}")

            # ... and if we should terminate, abort.
            if _should_terminate():
                logging.debug("usb1.TRANSFER_COMPLETED terminating")
                return

            # Otherwise, re-submit the transfer.
            transfer.submit()

        else:
            failed_out = status

    with usb1.USBContext() as context:

        # Grab a reference to our device...
        device = context.openByVendorIDAndProductID(VENDOR_ID, PRODUCT_ID)

        # ... and claim its bulk interface.
        device.claimInterface(0)

        # Submit a set of transfers to perform async comms with.
        active_transfers = []
        for _ in range(TRANSFER_QUEUE_DEPTH):

            # Allocate the transfer...
            transfer = device.getTransfer()
            transfer.setBulk(0x80 | BULK_ENDPOINT_NUMBER, TEST_TRANSFER_SIZE, callback=_transfer_completed, timeout=1000)

            # ... and store it.
            active_transfers.append(transfer)

        # Start our benchmark timer.
        start_time = time.time()

        # Submit our transfers all at once.
        for transfer in active_transfers:
            transfer.submit()

        # Tell Cynthion to start transmitting
        device.bulkWrite(COMMAND_ENDPOINT_NUMBER, [TestCommand.In])

        # Run our transfers until we get enough data.
        while not _should_terminate():
            context.handleEvents()

        # Figure out how long this took us.
        end_time = time.time()
        elapsed = end_time - start_time

        # Cancel all of our active transfers.
        for transfer in active_transfers:
            if transfer.isSubmitted():
                transfer.cancel()

        # If we failed out; tell Cynthion to stop transmitting and indicate it.
        if (failed_out):
            device.bulkWrite(COMMAND_ENDPOINT_NUMBER, [TestCommand.Stop])
            logging.error(f"Test failed because a transfer {_messages[failed_out]}.")
            sys.exit(failed_out)

        bytes_per_second = total_data_exchanged / elapsed
        logging.info(f"Exchanged {total_data_exchanged / 1000000}MB total at {bytes_per_second / 1000000}MB/s.")

        # Tell Cynthion to stop transmitting
        device.bulkWrite(COMMAND_ENDPOINT_NUMBER, [TestCommand.Stop])


# - OUT Speed Test ------------------------------------------------------------

def run_out_speed_test():
    """ Runs a simple OUT speed test, and reports throughput. """

    #test_data = bytearray([x % 256 for x in range(512)])
    test_data = bytearray([x % 256 for x in range(64)])
    total_data_exchanged = 0
    failed_out = False

    def _should_terminate():
        """ Returns true iff our test should terminate. """
        return (total_data_exchanged > TEST_DATA_SIZE) or failed_out


    def _transfer_completed(transfer: usb1.USBTransfer):
        """ Callback executed when an async transfer completes. """
        nonlocal total_data_exchanged, failed_out

        status = transfer.getStatus()

        # If the transfer completed.
        if status in (usb1.TRANSFER_COMPLETED,):

            # Count the data exchanged in this packet...
            total_data_exchanged += transfer.getActualLength()
            logging.info(f"usb1.TRANSFER_COMPLETED: {total_data_exchanged} bytes")

            # ... and if we should terminate, abort.
            if _should_terminate():
                logging.info("usb1.TRANSFER_COMPLETED terminating")
                return

            # Otherwise, re-submit the transfer.
            transfer.submit()

        else:
            failed_out = status

    with usb1.USBContext() as context:

        # Grab a reference to our device...
        device = context.openByVendorIDAndProductID(VENDOR_ID, PRODUCT_ID)

        # ... and claim its bulk interface.
        device.claimInterface(0)

        # Submit a set of transfers to perform async comms with.
        active_transfers = []
        for _ in range(TRANSFER_QUEUE_DEPTH):

            # Allocate the transfer...
            transfer = device.getTransfer()
            transfer.setBulk(0x00 | BULK_ENDPOINT_NUMBER, test_data, callback=_transfer_completed, timeout=1000)

            # ... and store it.
            active_transfers.append(transfer)

        # Start our benchmark timer.
        start_time = time.time()

        # Submit our transfers all at once.
        for transfer in active_transfers:
            transfer.submit()

        # Tell Cynthion to start transmitting
        device.bulkWrite(COMMAND_ENDPOINT_NUMBER, [TestCommand.In])

        # Run our transfers until we get enough data.
        while not _should_terminate():
            context.handleEvents()

        # Figure out how long this took us.
        end_time = time.time()
        elapsed = end_time - start_time

        # Cancel all of our active transfers.
        for transfer in active_transfers:
            if transfer.isSubmitted():
                transfer.cancel()

        # If we failed out; tell Cynthion to stop receiving and indicate it.
        if (failed_out):
            device.bulkWrite(COMMAND_ENDPOINT_NUMBER, [TestCommand.Stop])
            logging.error(f"Test failed because a transfer {_messages[failed_out]}.")
            sys.exit(failed_out)


        bytes_per_second = total_data_exchanged / elapsed
        logging.info(f"Exchanged {total_data_exchanged / 1000000}MB total at {bytes_per_second / 1000000}MB/s.")

        # Tell Cynthion to stop receiving
        device.bulkWrite(COMMAND_ENDPOINT_NUMBER, [TestCommand.Stop])


# - main entry point ----------------------------------------------------------

if __name__ == "__main__":
    configure_default_logging()
    logging.info("Running IN speed test...")
    try:
        run_in_speed_test()
    except Exception as e:
        logging.error(f"USB Bulk IN speed test failed: {e}")
    #logging.info("Running OUT speed test...")
    #try:
    #    run_out_speed_test()
    #except Exception as e:
    #    logging.error(f"USB Bulk OUT speed test failed: {e}")
