<!-- vim: set tw=80: -->

# Tempsys Scan

This repository contains the code for the scanning component of Tempsys. It will
read the advertising events emitted by the
[tempsys-firmware](https://github.com/fabiojmendes/tempsys-firmware) and publish
it to a mqtt broker.

## Configuration

A toml configuration file is necessary is necessary to provide the location for
the mqtt broker and also friendly names for each tempsys thermometer.
This file should be named `/etc/tempsys/config.toml` and a sample of what
it should look like is as follow:

```toml
devices = [
  { name = "TestDevice", addr = "AA:BB:CC:11:22:33", device_type = "Fridge" },
]

[mqtt]
id = "tempsys-scan"
host = "localhost"
port = 1883
username = "user"
password = "pass"
topic = "tempsys/temperature"
```

## Build

This code is meant to run on a raspberry pi device so a [cross](https://github.com/cross-rs/cross)
configuration is provided for easy of use.

To build it locally you will need to install the dependencies `libdbus-1-dev`
and `pkg-config`.

## WIP

- Breakdown code in modules
- Improve error handling
- Parse cli arguments?
