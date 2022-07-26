# Cpro IoT Connector for SysTec Scales 0.1.4

## Binary client

Connect to scales to retrieve live data

```sh
USAGE:
    scale-connector [OPTIONS] <ip>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --interval <interval>    Refresh interval in milliseconds, defaults to 1000 (1 second)
    -l, --log <log>              Set Log level
    -m, --mqtt <mqtt>            MQTT Server and port, e.g. 192.168.93.97:1500
    -p, --port <port>            Service port, defaults to "1234"

ARGS:
    <ip>    Hostname or IP Address
```

## Scale Connector Stack

In order to set up the scale stack, first run the build script:

`build_scale_compose_from_network.sh > auto-compose.yml`

This generates a docker-compose file that generates a stack that connects to all available scales in the network and gathers the data at the MQTT broker (service `mqtt-broker` for that matter).

`docker-compose -f auto-compose.yml up -d`
