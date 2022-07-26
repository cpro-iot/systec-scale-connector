# Systec Scale Connector

This repo contains docker-compose files to implement a IoT stack in order to connect systec scales

## Scale Connector Stack

In order to set up the scale stack, first run the build script:

`build_scale_compose_from_network.sh > auto-compose.yml`

This generates a docker-compose file that generates a stack that connects to all available scales in the network and gathers the data at the MQTT broker (service `mqtt-broker` for that matter).

`docker-compose -f auto-compose.yml up -d`

For more information on how to configure the scale connector, look at the Cpro Git
