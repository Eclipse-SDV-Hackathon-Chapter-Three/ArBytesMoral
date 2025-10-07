# CARLA Provider

The **CARLA Provider** demonstrates a practical bridge between CARLA and KUKSA Databroker (on the same or different machines). The Provider connects to a CARLA simulator, starts a Traffic Manager and spawns a vehicle with an autopilot driving it. Moreover it gently changes the weather over time. It reads back the wetness and pusblishes it to KUKSA Databroker as VSS conform value.

## Feature #1

Sets up CARLA simulation: connects, spawns a vehicle, enables autopilot, manipulates weather conditions and steps the sim every 50 ms.

## Feature #2

Reads CARLA weather wetness and maps it to the VSS signal Vehicle.Exterior.Humidity. Publishes Vehicle.Exterior.Humidity to the KUKSA Databroker once per second (non-blocking).
Reads CARLA vehicle velocity and maps it to the VSS signal Vehicle.Speed. Publishes Vehicle.Speed to the KUKSA Databroker once per second (non-blocking).
Reads CARLA GNSS location and maps it to the VSS signals Vehicle.CurrentLocation.(Longitude, Latitude, Altitude). Publishes Vehicle.CurrentLocation.(Longitude, Latitude, Altitude) to the KUKSA Databroker once per second (non-blocking).

## Feature #3

Keeps CARLA and KUKSA in sync while running, and shuts down cleanly on Ctrl+C.
