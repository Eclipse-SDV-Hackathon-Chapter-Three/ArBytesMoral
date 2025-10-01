#!/bin/sh

cd mqtt_kuksa_provider
# needs to run sudo because ank-agent runs as root user and 
# pulls from root container image storage
sudo podman build -t localhost/mqtt-kuksa-provider:latest .
cd ..