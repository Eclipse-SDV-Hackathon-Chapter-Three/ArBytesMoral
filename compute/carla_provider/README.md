# MQTT Kuksa Provider TODO

The **MQTT Kuksa Provider** acts as a bi-directional bridge between the Eclipse Mosquitto MQTT Broker and the Eclipse Kuksa DataBroker. It facilitates seamless communication between the MCU node and the centralized vehicle data model by translating MQTT messages to VSS signals and vice versa.

## Feature #1

Listen to MQTT topic mcu/temperature and transmit the temperature data on the corresponding VSS signal Vehicle.Cabin.HVAC.AmbientAirTemperature.

## Feature #2

Retrieves the VSS Signal Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide.Color and transmit the value on MQTT topic compute/color.

# CARLA Provider

The **CARLA Provider** demonstrates a practical bridge between CARLA and KUKSA Databroker (on the same or different machines). The Provider connects to a CARLA simulator, starts a Traffic Manager and spawns a vehicle with an autopilot driving it. Moreover it gently changes the weather over time. It reads back the wetness and pusblishes it to KUKSA Databroker as VSS conform value.

## Feature #1

Talks to CARLA: connects, spawns a vehicle, enables autopilot, and steps the sim every 50 ms.

## Feature #2

Reads CARLA weather wetness and maps it to the VSS signal Vehicle.Exterior.Humidity. Publishes Vehicle.Exterior.Humidity to the KUKSA Databroker once per second (non-blocking).

## Feature #3

Keeps CARLA and KUKSA in sync while running, and shuts down cleanly on Ctrl+C.
