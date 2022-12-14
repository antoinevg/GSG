<!DOCTYPE html>
<html lang="en">
<head>
    <link rel="stylesheet" href="./rfp.css">
    <style>
        body { padding: 100px; }
        img { max-width: 1024; }
    </style>
    <meta charset="utf-8">
    <meta name="description" content="Luna Initial Design Specification">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>rfp.html</title>
</head>

<body>

# [FaceDancer: Implement Initial Design Specification / RFP #68](https://github.com/greatscottgadgets/luna/issues/68)

## Main Task

Create an initial design document describing a FaceDancer backend for LUNA.

This should:

* [ ] Include plans for the core backend.
* [ ] Accommodate future FPGA acceleration, where possible.
* [ ] Accommodate fancy USBProxy enhancements.



---

## Terms

* Shall = requirement
* Will = facts or declaration of purpose
* Should = goal



---

## 1 Introduction

The Facedancer software is a host-side Python module for the control of simple hardware devices that act as "remote-controlled" USB controllers.

### 1.1 Data flow

Adding support to Facedancer for a new remote-controlled USB device ("the device") requires the implementation of a number of major components on both the controlling host ("the host") and the device being controlled.

This proposal will cover the requirements for the development of a Facedancer backend for the [Great Scott Gadgets Luna](https://greatscottgadgets.com/luna/).

Understanding the relationships between the major components of Facedancer can best be visualized in terms of the data flow between them:

![Diagram: Data Flow](dataflow/luna.svg)

The host side can be broken down into the following major components:

1. The *Luna Facedancer Backend*: `facedancer.git:/facedancer/backends/moondancer.py`
2. The *Luna host-side Python Library*: `luna.git:/host/`
3. The *Luna host-side CLI*: : `luna.git:/host/`
3. The *Host-side Command Serialization Library*: `libgreat.git:/host/pygreat/`
4. The *Host-side Transport Library*: `libgreat.git:/host/pygreat/`

The device side consists of the major components:

1. The *Device-side Transport Library*: `libgreat-rs.git:/firmware/drivers/`
2. The *Device-side Command Serialization Library*: `libgreat-rs.git:/firmware/drivers/`
3. The *Luna Device API*: `luna.git:/luna/firmware/soc/classes/`
4. The *Luna SoC Firmware*: `luna.git:/luna/firmware/soc/`
5. The *Luna SoC Peripheral Drivers*: `libgreat-rs.git:/firmware/platform/soc/drivers/`
6. The *Luna SoC Gateware*: `luna.git:/luna/gateware/`
7. The *Luna Physical Hardware*: `luna.git:/hardware/`



---

## 2 Host Side Implementation

There is an existing host side implementation in Python which targets the [Greatt Scott Gadgets GreatFET](https://greatscottgadgets.com/greatfet/). We should be able to re-use much of the code with only minor modification and re-organization of the existing codebase.

This section exists primarily to serve as a reference to how this code is currently organized.

<!--
![Diagram: Facedancer](facedancer/top_facedancer.svg)
 TODO split into individual diagrams for each section
 -->

The primary use-case for Facedancer is writing small Python scripts to automate the behaviour of Facedancer "Applets". Facedancer Applets are Python classes that encapsulate the behaviour of a given USB Device to be emulated.

For example:

![Diagram: Facedancer Rubber Ducky Example](facedancer/facedancer_git.examples_rubber_ducky.svg)

Currently, there are two applets available:

* [`FTDIDevice`](https://github.com/greatscottgadgets/Facedancer/blob/master/facedancer/devices/ftdi.py) - Emulated FTDI USB Serial device.
* [`USBKeyboardDevice`](https://github.com/greatscottgadgets/Facedancer/blob/master/facedancer/devices/keyboard.py) - Simple USB Keyboard device.

There are also a number of legacy applets available that we will talk about more in [Section 5.5]().

This user-facing scripting environment is abstracted away from any given Facedancer hardware allowing support to be extended for new devices.

The entry-point for adding new device support to Facedancer is via the implementation of a Luna Facedancer Backend.


### 2.1 Luna Facedancer Backend

A Facedancer host backend ("the facedancer backend) is responsible for servicing the API surface exposed to Facedancer applets. ("the Facedancer applet API")

![Diagram: Luna Facedancer Backend](facedancer/facedancer_git.backends_moondancer.svg)

As such, it is primarily responsible for enacting the following functionality on the device being controlled by Facedancer:

* Initialize the USB device, configuration, interface and endpoint descriptors of the device.
* Read data from endpoints.
* Write data to endpoints.


### 2.2 Luna host-side Python Library

Great Scott Gadgets device commands are organized into a simple class/verb scheme to define a transport-independent format ("great communications protocol") for communication with a host-connected device.

![Diagram: Luna host-side Python Library](facedancer/greatfet_git.host_greatfet.svg)


### 2.3 Command Serialization Library

Luna uses USB as the transport for "great communications protocol" messages.

![Diagram: Command Serialization Library](facedancer/libreat_git.pygreat.svg)


### 2.4 Transport Library

TODO

<!-- ![Diagram: Command Transport Library](facedancer/.svg) -->



### 2.5 Main Tasks:

* [ ] Derive a new Luna Facedancer Backend from [facedancer.git:facedancer/backends/greatdancer.py](https://github.com/greatscottgadgets/Facedancer/blob/master/facedancer/backends/greatdancer.py)
* [ ] Derive a new Luna API Library from [greatfet.git host/greatfet/](https://github.com/greatscottgadgets/greatfet/tree/master/host/greatfet/)
* [ ] Re-use the Command Serialization and Transport Library from: [libgreat.git host/pygreat/comms_backends/usb.py](https://github.com/greatscottgadgets/libgreat/blob/master/host/pygreat/comms_backends/usb.py)



---

## 3 Device Side Firmware Implementation

The primary function of the Device Firmware is to implement the functionality defined by the "great communications protocol" messages between Facedancer and the device.

Additional functionality includes logging, error handling, reset control, device heartbear and SoC firmware updates.

![Diagram: Firmware Structure](structure/luna_git.firmware.svg)

The device firmware shall be implemented in Rust following established community guidelines for Embedded Rust development.


### 3.1 Command Serialization and Transport library

The USB serialization libary is responsible for marshalling "great communication protocol" classes/verbs between the device firmware and the USBDeviceController peripheral API.

#### Reference Implementation

[libgreat.git:/firmware/drivers/comms/](https://github.com/greatscottgadgets/libgreat/tree/master/firmware/drivers/comms/)


### 3.2 Luna Device API

The main classes that should be supported by the Luna hardware are:

* `0x105 usbhost`
    - remote control over Luna’s USB ports in host mode, for e.g. FaceDancer
* `0x120 moondancer` (new class)
    - remote control over Luna’s USB ports in device mode, for e.g. FaceDancer
* `0x10F usbproxy`
    - Firmware functionality supporting USBProxy

Additional classes of interest are:

* `0x107 glitchkit_usb`
    - control over functionality intended to help with timing USB fault injection
* `0x113 usb_analysis`
    - functionality for USB analysis e.g. with Rhododendron

Specifically, support should be implemented for the following operations:

* Configure descriptors for USBDeviceController peripheral
* Read data from endpoints
* Write data to endpoints
* TODO

#### Reference Implementation

[greatfet.git:/firmware/greatfet_usb/classes/greatdancer.c](https://github.com/greatscottgadgets/greatfet/tree/master/firmware/greatfet_usb/classes/greatdancer.c)


### 3.3 Luna SoC Firmware

* Logging
* Error handling
* Reset control
* Device heartbeat
* SoC firmware update

#### Reference Implementation

[greatfet.git:/firmware/greatfet_usb/](https://github.com/greatscottgadgets/greatfet/tree/master/firmware/greatfet_usb/)

<!--
![Diagram: GreatFET](greatfet/top_greatfet.svg)
TODO split into individual diagrams for sections above
-->

### 3.4 Luna SoC Peripheral Drivers

Drivers need to be implemented for the following SoC peripherals:

* `amaranth_stdio.serial.AsyncSerial`
* `lambdasoc.periph.timer.TimerPeripheral`
* `luna.gateware.interface.flash.ECP5ConfigurationFlashInterface`
* `luna.gateware.usb2.USBDeviceController`
* `luna.gateware.soc.GpioPeripheral`

Peripheral drivers are organized across three crates:

* `lunasoc-pac`  - A peripheral register access API generated by `svd2rust`
* `amaranth-hal` - Concrete implementations of the device-independent `embedded-hal` traits for the peripherals.
* `lunasoc-hal`  - Re-exports of `libgreat-hal` and any peripherals that don't belong in amaranth-soc.

> Note: The embedded-hal drivers can live in `lunasoc-hal` during development but with the understanding that this code can potentially run on any SoC built using the `amaranth-soc` and `lambdasoc` libraries. As such, it would be beneficial to both GSG and the Amaranth community if it could find a forever home upstream.

#### Reference Implementation

[libgreat.git:/firmware/platform/lpc43xx/](https://github.com/greatscottgadgets/libgreat/tree/master/firmware/platform/lpc43xx)


### 3.5 Main Tasks

* [ ] Implement support for the "great communication protocol" wire format using the [`serde`](https://serde.rs/data-format.html) crate.
* [ ]



---

## 4 Device Side Gateware Implementation

![Diagram: Device SoC](structure/luna_git.gateware_soc.svg)

###  Reference Implementation

[simplesoc.py]()


### 4.2 Main tasks

* [ ] Update the `simplesoc.py` placeholder to the latest versions of the `amaranth-soc` and `lambdasoc` libraries.

* Create additional peripherals as required:
    - [ ] GpioPeripheral
    - [ ] SPIFlashPeripheral ?
* Wire up additional peripherals as required:
    - [ ] GpioPeripheral
    - [ ] SPIFlashPeripheral / ECP5ConfigurationFlashInterface ?
    - [ ] USBDeviceController
    - [ ] HyperRAMInterface
* [ ] Implement SVD export for SoC designs
    - Bearing in mind that this too would benefit from finding a cosy home upstream

Gateware shall be implemented in [Amaranth](https://github.com/amaranth-lang/amaranth) with the use of external libraries such as `Amaranth-SoC` and `LambdaSoC` where possible.



---

## 5 Luna Host-side CLI

Luna should have a host-side command line interface which enables Luna owners to interact with the device hardware.

The following conventions shall be respected:

* Commands will be implemented as a single keyword containing no preceding or interceding dash or underscore tokens to distinguish them from flags.
* Flags will be distinguished by a single dash (abbreviated flag name) and a double dash (expanded flag name).
* Abbreviated flag names shall use the first letter of their expanded variant. Where this leads to conflicts alternative wording should be explored for the expanded names.
* Abbreviated flag names shall be capitalized in order to further distinguish them from expanded flag names.
* Abbreviated flag names may be omitted for lesser-used commands (e.g. `--keep-files`)

#### Utility Commands will include:

* `info`: Displays information about connected Luna devices.
* `factory`: Restores the device to it's factory configuration state and applies any newly released firmware and gateware updates.
* `reset`: Performs a device reset.
* `shell`: Start up an interactive Python shell from which the Luna host-side Python Library can be called.

#### User Commands will include:

* TBD - basically any top-level (i.e. non-facedancer) Luna functionality we're currently calling from standalone scripts.

#### Bitstream Commands will include:

* `output`: Build and output a bitstream to the given file.
* `erase`: Clears the relevant FPGA's flash before performing other options.
* `upload`: Uploads the relevant design to the target hardware. Default if no options are provided.
* `flash`: Flashes the relevant design to the target hardware's configuration flash.

The following flags will be supported:

* `--dry-run, -D`: When provided as the only option; builds the relevant bitstream without uploading or flashing it.
* `--keep-files`: Keeps the local files in the default `build` folder.
* `--fpga`: Overrides build configuration to build for a given FPGA. Useful if no FPGA is connected during build.
* `--console`: Attempts to open a convenience 115200 8N1 UART console on the specified port immediately after uploading.

#### SoC Commands will include

* `upload`: Uploads the given SoC Firmware to the SoC.
* `flash`: Flashes the given SoC Firmware to the target hardware's configuration flash.
* `sdk`: Generates a developer SDK for programming the Luna SoC.

The following flags will be supported:

* `--format=svd`:
* `--format=c-header`: A C header file for this design's SoC will be printed to the stdout. Other options ignored.
* `--format=ld-script`: A linker script for design's SoC memory regions be printed to the stdout. Other options ignored.
* `--format=rust-pac`: A Rust Peripheral Access Crate (PAC) for this design's SoC will be generated in the default `build/` directory.

#### Global flags will include:

* `--help`: Display Luna CLI help.
* `--build-dir`: Override the default `build/` directory.
* `--` All flags following a naked double-dash will be passed to any tools invoked by the Luna CLI.
* TBD

#### Naming considerations

* Consider: `output` -> `bitstream`
* Resolve identical commands in separate contexts (e.g. `flash`)
    - Option: Use a class/verb structure for commands e.g. `luna bitstream flash` / `luna firmware flash`
    - Option: Use different command names e.g. `store` for bitstream, `flash` for firmware.

#### Reference Implementation:

* [`luna.git:/luna/__init__.py`](https://github.com/greatscottgadgets/luna/blob/main/luna/__init__.py)
* [`GreatFETArgumentParser`](https://github.com/greatscottgadgets/greatfet/blob/master/host/greatfet/utils.py#L137)

It's recommended that the design of `GreatFETArgumentParser` be adapted for general use, renamed to `GreatArgumentParser` and homed in `libgreat.git`.


---

## 6 Additional Features

### 6.1 Additional Facedancer Applets

Facedancer device emulations are referred to as "applets" (is this correct?) that can be found in the [facedancer.git:/facedancer/devices](https://github.com/greatscottgadgets/Facedancer/tree/master/facedancer/devices) directly.

There are currently Facedancer applets for:

* [`FTDIDevice`](https://github.com/greatscottgadgets/Facedancer/blob/master/facedancer/devices/ftdi.py) - Emulated FTDI USB Serial device.
* [`USBKeyboardDevice`](https://github.com/greatscottgadgets/Facedancer/blob/master/facedancer/devices/keyboard.py) - Simple USB Keyboard device.

There are also a number of Facedancer applets that live in the [facedancer.git:/legacy_applets/](https://github.com/greatscottgadgets/Facedancer/tree/master/legacy-applets) directory. If is due to API changes they should probably be refactored to the new API so that they be made available:

* `facedancer-edl`
* `facedancer-ftdi`
* `facedancer-host-enumeration`
* `facedancer-keyboard-interactive`
* `facedancer-keyboard`
* `facedancer-procontroller`
* `facedancer-serial`
* `facedancer-switchtas`
* `facedancer-umass`
* `facedancer-ums-doublefetch`

### 6.2 Support for Luna Pmod Ports

There exists an opportunity to make the GreatFET gpio-related commands available under Luna as well.

### 6.3 New Facedancer Applet Creation

Enabling new Facedancer Applet creation primarily requires that the Facedancer Applet API be documented.

### 6.4 USBProxy

TBD

### 6.5 USB Fuzzing

TBD

### 6.6 Main Tasks

* [ ] Bring Facedancer's legacy applets up to date
* [ ] Support GreatFET gpio-related commands
* [ ] Document the Facedancer applet API
* [ ] Specify USBProxy Support
* [ ] Implement USBProxy Support
* [ ] Specify USB Fuzzing Support
* [ ] Implement USB Fuzzing Support



---

## 7 Documentation

Primary documentation should be developed for:

* Getting started with Luna and Facedancer
* Facedancer applet reference
* Luna host-side API reference
* Luna device-side API reference (docs.rs)

Additional documentation:

* Luna Tutorials
* Facedancer applet API reference


### References

* [GreatFET Tutorials](https://greatscottgadgets.github.io/greatfet-tutorials/)
* [GreatFET Project documentation](https://greatfet.readthedocs.io/en/latest/)
* [Facedancer README](https://github.com/greatscottgadgets/Facedancer/blob/master/README.md)

External:

* [WikiLeaks - Facedancer User Guide](https://wikileaks.org/ciav7p1/cms/page_20873567.html)
* [Travis Goodspeed - Emulating USB Devices with Python](https://travisgoodspeed.blogspot.com/2012/07/emulating-usb-devices-with-python.html)
* [Hackaday - Hands-on: GreatFET](https://hackaday.com/2019/07/02/hands-on-greatfet-is-an-embedded-tool-that-does-it-all/)
* [Colin O'Flunn - USB Attacks and More with GreatFET](https://circuitcellar.com/research-design-hub/usb-attacks-and-more-with-greatfet/)



---

## 8 Code Organization

The `luna.git` repository should utilize GSG repository layout conventions with the following top-level structure:

```
/ci-scripts  - Scripts used by CI
/docs        - Documentation
/firmware    - Device firmware
/hardware    - Device hardware (deprecated: see note below)
/gateware    - Device gateware
/host        - Host-side software
/tools       - Tools
```

> Note: The hardware release cycle operates on a different time-scale to software release cycles. Ideally device hardware should live in its own repository.

To reduce the maintenance burden across the larger Great Scott Gadgets codebase it's proposed to make some minor changes to the organization of the greatfet.git and libgreat.git repositories.

### 8.1 Facedancer "Great Communication Protocol" class id:

Class ids currently live in:

https://github.com/greatscottgadgets/greatfet/blob/master/docs/source/greatfet_classes.rst

1. Move greatfet_classes.rst from greatfet.git into libgreat.git
2. Add class id for Luna

Also see: https://gsg.atlassian.net/wiki/spaces/MEETINGS/pages/2438824143/2022-12-06+Engineering+Work+Session

### 8.2 Common code from greatfet.git

There may be some common code in the greatfet.git python host code that needs to be made available to Luna. This may also need to move to the libreat.git repository.

This includes:

* https://github.com/greatscottgadgets/greatfet/blob/master/host/greatfet/commands/greatfet_usb_capture.py
* TODO

### 8.3 Conformance to GSG project directory layout conventions

For the `luna.git` repository the following changes should be made:

* `luna.git:/luna/gateware` -> `luna.git:/gateware`
* `luna.git:/luna/` -> `luna.git:/host/luna`

### 8.4 Git submodules aka "Kill It With Fire!"

A strategy of escalation for dependencies that exist as git submodules:

1. Prefer the packaged library form.
2. If a packaged library does not exist fork the project and submit an upstream PR.
3. Switch to the packaged library from our fork while waiting for the PR process to play out.
4. If upstream can/will not accept the PR consider moving to an alternate dependency more aligned with our use-case.
5. If another dependency does not exist there are two choices:
    - Maintain our fork indefinitely. Sometimes this is not a big deal and upstream positions often shift given time, congeniality and reflection.
    - Vendor the forked code into `libgreat.git` remembering to include licenses and a great-full acknowledgement.

This strategy is hard work that can cost a couple days work but we pay it forward because, in Glorious Socialist Federation of Neighbours, upstream project take care of you!



---

## 9 Open Questions

* [ ] how do we want to manage SoC firmware uploads to SPI flash
    - appended to the bitstream uploaded to SPI flash?
    - SoC bios takes over SPI flash after ECP5 boot and uses a dedicated region for separate upload?
    - Which of the many available tools should Luna CLI use if we support a separate flash operations?
    - other ?

* [ ] do we want to support these additional usb-related classes for Luna:
    - `0x107 glitchkit_usb` - control over functionality intended to help with timing USB fault injection
    - `0x113 usb_analysis`  - functionality for USB analysis e.g. with Rhododendron

* [ ] are there any constraints on the "Great Communications Protocol" implementation that will place limits on what we can do with Luna vs GreatFET?

* [ ] `libgreat-rs.git` -> `libgreat.git/firmware-rs/greatsoc-hal, greatsoc-pac` etc. or somesuch ?
    - the problem with having two repo's that both start with libgreat is that it may not be clear which one to look in?
    - on the other hand, nervous about putting the rust crate into libgreat because more complex CI and releases might ensue ?


* [ ] Should we implement the main device firmware in a pure Rust `no_std` environment without the use of the `alloc` feature?
    - `alloc` comes with its own share of problems in the form of memory fragmentation and unpredictable latency.
    - are there any high-alloc areas of the code that can't be handled by e.g. the `heapless` crate.



---

## 10 Diagrams

### Dataflow Comparison of GreatFET and Luna

![Diagram: Dataflow Comparison of GreatFET and Luna](dataflow/top_dataflow.svg)

### Device-side Structural Decomposition

![Diagram: Device-side Structural Decomposition](structure/top_structure.svg)

### Facedancer Call Graph

![Diagram: Facedancer Call Graph](facedancer/top_facedancer.svg)

### GreatFET Firmware Call Graph

![Diagram: GreatFET Firmware Call Graph](greatfet/top_greatfet.svg)


---

## 11 Appendix

### Links to Useful Documentation

* [GreatFET Documentation](https://greatfet.readthedocs.io/en/latest/)
    - [Verb Signatures](https://greatfet.readthedocs.io/en/latest/libgreat_verb_signatures.html)
    - [Classes](https://greatfet.readthedocs.io/en/latest/greatfet_classes.html)

External:

* [WikiLeaks - Facedancer User Guide](https://wikileaks.org/ciav7p1/cms/page_20873567.html)
* [Writing a Serde data format](https://serde.rs/data-format.html)
* [Beyond Logic - USB in a NutShell](https://www.beyondlogic.org/usbnutshell/)

### Repositories

* https://github.com/antoinevg/GSG-private/
* https://github.com/antoinevg/luna
* https://github.com/antoinevg/Facedancer
* https://github.com/greatscottgadgets/libgreat
* https://github.com/greatscottgadgets/greatfet

### Issues

* [FaceDancer: Implement Initial Design Specification / RFP #68](https://github.com/greatscottgadgets/luna/issues/68)
* [Facedancer SoC #151](https://github.com/greatscottgadgets/luna/issues/151)
* [LUNA support for Facedancer #59](https://github.com/greatscottgadgets/Facedancer/issues/59)


</body>
</html>
