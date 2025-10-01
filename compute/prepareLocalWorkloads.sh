#!/bin/sh

cd mqtt_kuksa_provider
# needs to run sudo because ank-agent runs as root user and 
# pulls from root container image storage
sudo podman build -t localhost/mqtt-kuksa-provider:latest .
cd ..

## VEHICLE DATA ACCESSOR ####################################
cd car_mate_io
# needs to run sudo because ank-agent runs as root user and
# pulls from root container image storage
sudo podman build -t localhost/car-mate-io:latest .
cd ..
#############################################################

## VEHICLE DATA ACCESSOR ####################################
cd car_mate_agents
# needs to run sudo because ank-agent runs as root user and
# pulls from root container image storage
sudo podman build -t localhost/car-mate-agents:latest .
cd ..
#############################################################