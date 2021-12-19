# Lucia

A tool for controlling Philips Hue lights from the comfort of a terminal. The aim is to cover
most of the functionality exposed by the [Hue API][1], but perhaps in the future to also
augment it and/or support devices by other vendors.

```text
lucia 0.1.0

USAGE:
    lucia <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    configure    Connect to a Hue Bridge address and attempts to pair with it
    devices      Print all lights known by the pre-configured bridges
    discover     Print all discovered Hue Bridge devices
    help         Print this message or the help of the given subcommand(s)
    light        Set properties of a list of light sources identified by their id

```

## Set Up

Use the `configure` subcommand and give it the IP address of the Hue Bridge. Follow the
instructions in the output. You can list all local bridges with the `discover` subcommand,
in case you don't know their addresses. The setup will create a `lucia.json` file in your
home directory with information needed for calling the API.

You may verify that everything is working by running the `devices` subcommand.

[1]: https://developers.meethue.com/develop/hue-api
