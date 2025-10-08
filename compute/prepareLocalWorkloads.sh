#!/bin/sh
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

## MQTT KUKSA PROVIDER #####################################
cd mqtt_kuksa_provider
# needs to run sudo because ank-agent runs as root user and 
# pulls from root container image storage
sudo podman build -t localhost/mqtt-kuksa-provider:latest .
cd ..

## IO CARMATE ####################################
cd car_mate_io
# needs to run sudo because ank-agent runs as root user and
# pulls from root container image storage
sudo podman build -t localhost/car-mate-io:latest .
cd ..
#############################################################

## AGENTS CARMATE ####################################
cd car_mate_agents
# needs to run sudo because ank-agent runs as root user and
# pulls from root container image storage
sudo podman build -t localhost/car-mate-agents:latest .
cd ..
#############################################################

## VEHICLE DATA ACCESSOR ####################################
cd vehicle_data_accessor
# needs to run sudo because ank-agent runs as root user and 
# pulls from root container image storage
sudo podman build -t localhost/vehicle-data-accessor:latest .
cd ..

## CARLA PROVIDER ###########################################
cd carla_provider
# needs to run sudo because ank-agent runs as root user and
# pulls from root container image storage
sudo podman build -t localhost/carla-provider:latest .
cd ..
#############################################################