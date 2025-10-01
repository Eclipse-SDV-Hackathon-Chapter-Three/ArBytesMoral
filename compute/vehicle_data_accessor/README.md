# MQTT Kuksa Provider TODO

The **MQTT Kuksa Provider** acts as a bi-directional bridge between the Eclipse Mosquitto MQTT Broker and the Eclipse Kuksa DataBroker. It facilitates seamless communication between the MCU node and the centralized vehicle data model by translating MQTT messages to VSS signals and vice versa.

## Feature #1

Listen to MQTT topic mcu/temperature and transmit the temperature data on the corresponding VSS signal Vehicle.Cabin.HVAC.AmbientAirTemperature.

## Feature #2

Retrieves the VSS Signal Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide.Color and transmit the value on MQTT topic compute/color.