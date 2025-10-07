#!/bin/bash
# *******************************************************************************
# Copyright (c) 2025 Eclipse Foundation and others.
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# SPDX-License-Identifier: Apache-2.0
# *******************************************************************************

sudo podman kill -a

# List of workloads to delete in the desired order
WORKLOADS=(
  Ankaios_Dashboard
  carlaprovider
  vehilceDataAccessor
  CarMateAgents
  CarMateIO
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
