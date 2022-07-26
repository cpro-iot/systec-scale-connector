#!/bin/bash

function log() { echo "$@" 1>&2; }

log "Find online Systec scales..."

# Get all systec MACs
SCALES=(`arp -na | grep 48:c8:b6 | awk '{ gsub(/\(|\)/,""); print $2 }'`)

# Scan MAC IPs for open port 1234...
OPEN_SCALES=(`nmap -p 1234 ${SCALES[@]} -oG - | grep open | awk '{ print $2}'`)

log "Build stack for ${#OPEN_SCALES[@]}/${#SCALES[@]} scales"
cat scale-connectors.template.yml
for SCALE in "${OPEN_SCALES[@]}"; do
	IFS=. read one two three ID <<< ${SCALE}
	cat << EndOfService
  scale_${ID}:
    <<: *function
    container_name: scale_${ID}
    environment:
      HOST: ${SCALE}
EndOfService
done
