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

### Eclipse Zenoh Router

The Zenoh Router is a core component of the Zenoh protocol that efficiently routes data between clients and subnetworks in diverse network topologies, enabling seamless data exchange and discovery in heterogeneous and geo-distributed environments. It supports low-latency, high-throughput communication with minimal overhead, making it suitable for resource-constrained devices and complex systems like automotive, industrial IoT, and robotics. Zenoh routers also provide adaptive routing, distributed edge storage, and integration capabilities, facilitating scalable, reliable, and flexible data sharing from edge to cloud. It is necessary to allow communication via Eclipse uProtocol with Eclipse Zenoh as uTransport.

### MQTT Kuksa Provider

The **MQTT Kuksa Provider** acts as a bi-directional bridge between the Eclipse Mosquitto MQTT Broker and the Eclipse Kuksa DataBroker. It facilitates seamless communication between the MCU node and the centralized vehicle data model by translating MQTT messages to VSS signals and vice versa.

Further details can be found in the related README.md.

### CarMateIO

The **CarMateIO** acts as an interface to the driver by hosting an webapp where the driver can talk with his car or give commands to gain information on its surroundings. The main Feature is Speech to Text (STT) and Text to Speech (TTS). The App talks to **CarMateAgents** that connects to a LLM and the **Vehicle Data Accessor**. 

Further details can be found in the related README.md.

### CarMateAgents

The **CarMateAgents** acts as an interface between the **CarMateIO** to a LLM and the **Vehicle Data Accessor**. It consists of an supervisor agent that controls different agents using tool. For example the generic openai api, an online weather service or vehicle data provided via **Vehicle Data Accessor**.

Further details can be found in the related README.md.




### Vehicle Data Accessor

The **Vehicle Data Accessor** connects the Eclipse Kuksa DataBroker with **Eclipse uProtocol** and **Eclipse Zenoh**, enabling the CarMate application to interact seamlessly with the VSS (Vehicle Signal Specification) data model.

Further details can be found in the related README.md.

### CARLA Provider

The **CARLA Provider** demonstrates a practical bridge between CARLA and KUKSA Databroker (on the same or different machines). The Provider connects to a CARLA simulator, starts a Traffic Manager and spawns a vehicle with an autopilot driving it. Moreover it gently changes the weather over time. It reads back the wetness and pusblishes it to KUKSA Databroker as VSS conform value.

Further details can be found in the related README.md.
