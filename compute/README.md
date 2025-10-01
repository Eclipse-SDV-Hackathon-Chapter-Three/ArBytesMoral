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

The workloads of the compute node are defined in the ankaios.yaml file. Some of these workloads need to be prepared locally before they can be execited. To prepare the workloads,run the repareLocalWorkloads.sh script.

### Eclipse Kuksa Databroker

The **Eclipse Kuksa DataBroker** serves as the central data management component, providing read and write access to the standardized Vehicle Signal Specification (VSS) data model. It enables key functionalities like ingesting vehicle sensor data and serving applications that interact with the vehicle's digital twin.

### Eclipse Mosquitto MQTT Broker

The **Eclipse Mosquitto MQTT Broker** provides the messaging service that acts as the communication backbone between components like the MCU board and the vehicle's compute node. It operates as a lightweight, standards-compliant MQTT implementation. Authentication is disabled.

### MQTT Kuksa Provider

The **MQTT Kuksa Provider** acts as a bi-directional bridge between the Eclipse Mosquitto MQTT Broker and the Eclipse Kuksa DataBroker. It facilitates seamless communication between the MCU node and the centralized vehicle data model by translating MQTT messages to VSS signals and vice versa.

Further details can be found in the related README.md.

### CARLA Provider

The **CARLA Provider** demonstrates a practical bridge between CARLA and KUKSA Databroker (on the same or different machines). The Provider connects to a CARLA simulator, starts a Traffic Manager and spawns a vehicle with an autopilot driving it. Moreover it gently changes the weather over time. It reads back the wetness and pusblishes it to KUKSA Databroker as VSS conform value.