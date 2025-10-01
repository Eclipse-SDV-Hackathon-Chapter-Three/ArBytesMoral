# Vehicle Data Accessor

The **Vehicle Data Accessor** connects the Eclipse Kuksa DataBroker with **Eclipse uProtocol** and **Eclipse Zenoh**, enabling the CarMate application to interact seamlessly with the VSS (Vehicle Signal Specification) data model.

## Feature #1

Retrieve the relevant VSS Signals Vehicle.Exterior.Humidity and Vehicle.Cabin.HVAC.AmbientAirTemperature and transmit the values on uProtocol with Zenoh.

## Feature #2

Transfer the Ambient Color to be set received via Uprotocol and Zenoh to the VSS Signal Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide.Color.