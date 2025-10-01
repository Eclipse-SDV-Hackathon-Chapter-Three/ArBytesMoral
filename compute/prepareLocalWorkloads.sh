#!/bin/sh

## MQTT KUKSA PROVIDER #####################################
cd mqtt_kuksa_provider
# needs to run sudo because ank-agent runs as root user and 
# pulls from root container image storage
sudo podman build -t localhost/mqtt-kuksa-provider:latest .
cd ..

## IO CARMATE ####################################
cd car_mate_io
# needs to run sudo because ank-agent runs as root user and
# pulls from root container image storage
sudo podman build -t localhost/car-mate-io:latest .
cd ..
#############################################################

## AGENTS CARMATE ####################################
cd car_mate_agents
# needs to run sudo because ank-agent runs as root user and
# pulls from root container image storage
sudo podman build -t localhost/car-mate-agents:latest .
#############################################################

## VEHICLE DATA ACCESSOR ####################################
cd vehicle_data_accessor
# needs to run sudo because ank-agent runs as root user and 
# pulls from root container image storage
sudo podman build -t localhost/vehicle-data-accessor:latest .
cd ..

## CARLA PROVIDER ###########################################
cd carla_provider
# needs to run sudo because ank-agent runs as root user and
# pulls from root container image storage
sudo podman build -t localhost/carla-provider:latest .
cd ..
#############################################################