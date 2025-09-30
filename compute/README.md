# Compute Node

This document provides an overview of the compute node in the context of the SDV Lab Challenge.

## Overview

Eclipse Ankaios is used as workload and container orchestration. See https://eclipse-ankaios.github.io/ankaios/latest/ for further information about Eclipse Ankaios.

Assumption is that there is an ank-server and ank-agent running on the compute ecu.

sudo systemctl start ank-server
sudo systemctl start ank-agent

Make sure that the local agent name corresponds to agent_compute in /etc/ankaios/ank-agent.conf. Otherwise the ank-server cannot find the corresponding ank-agent.

Then the related workloads can be started by

ank -k apply ankaios.yaml

## Workloads

### Eclipse Kuksa Databroker

The **Eclipse Kuksa DataBroker** serves as the central data management component, providing read and write access to the standardized Vehicle Signal Specification (VSS) data model. It enables key functionalities like ingesting vehicle sensor data and serving applications that interact with the vehicle's digital twin.

### Eclipse Mosquitto MQTT Broker

The **Eclipse Mosquitto MQTT Broker** provides the messaging service that acts as the communication backbone between components like the MCU board and the vehicle's compute node. It operates as a lightweight, standards-compliant MQTT implementation. Authentication is disabled.
