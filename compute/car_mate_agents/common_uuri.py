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