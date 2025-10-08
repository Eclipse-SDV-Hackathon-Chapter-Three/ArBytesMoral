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

import logging

import zenoh
from uprotocol.v1.uri_pb2 import UUri

# Configure the logging
logging.basicConfig(level=logging.DEBUG, format='%(asctime)s - %(levelname)s - %(message)s')

def create_method_uri():
    return UUri(authority_name="voice-command", ue_id=18, ue_version_major=1, resource_id=7001)

def get_zenoh_config():
    conf = zenoh.Config()
    conf.from_file(path="zenoh.json")
    return conf

# Initialize Zenoh with default configuration
def get_zenoh_default_config():
    # Create a Zenoh configuration object with default settings
    config = zenoh.Config()

    return config