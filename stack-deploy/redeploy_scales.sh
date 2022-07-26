#!/bin/bash
docker-compose -f scale-connectors.yml down
docker rmi docker.cpronect.de/scale-connector
docker-compose -f scale-connectors.yml up -d


