"""
SPDX-FileCopyrightText: 2024 Contributors to the Eclipse Foundation

See the NOTICE file(s) distributed with this work for additional
information regarding copyright ownership.

This program and the accompanying materials are made available under the
terms of the Apache License Version 2.0 which is available at

    http://www.apache.org/licenses/LICENSE-2.0

SPDX-License-Identifier: Apache-2.0
"""

import json
import logging
from enum import Enum

import zenoh
from uprotocol.v1.uri_pb2 import UUri

# Configure the logging
logging.basicConfig(level=logging.DEBUG, format='%(asctime)s - %(levelname)s - %(message)s')


class ExampleType(Enum):
    PUBLISHER = "publisher"
    SUBSCRIBER = "subscriber"
    RPC_SERVER = "rpc_server"
    RPC_CLIENT = "rpc_client"


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
