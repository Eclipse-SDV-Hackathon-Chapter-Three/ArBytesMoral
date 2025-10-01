#!/bin/bash

# List of workloads to delete in the desired order
WORKLOADS=(
  Ankaios_Dashboard
  carlaprovider
  vehilceDataAccessor
  mqtt2kuksa
  mqtt
  databroker
)

# Delete workloads in sequence
for wl in "${WORKLOADS[@]}"; do
  echo "Deleting workload: $wl ..."
  ank delete workload "$wl"
  
  # Check if the command succeeded
  if [ $? -eq 0 ]; then
    echo "Successfully deleted: $wl"
  else
    echo "Failed to delete $wl"
    # Optional: stop the script if a deletion fails
    # exit 1
  fi
done

echo "All workloads processed."
