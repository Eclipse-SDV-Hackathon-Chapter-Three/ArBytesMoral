"""
SPDX-FileCopyrightText: 2024 Contributors to the Eclipse Foundation

See the NOTICE file(s) distributed with this work for additional
information regarding copyright ownership.

This program and the accompanying materials are made available under the
terms of the Apache License Version 2.0 which is available at

    http://www.apache.org/licenses/LICENSE-2.0

SPDX-License-Identifier: Apache-2.0
"""

import asyncio

from uprotocol.communication.inmemoryrpcclient import InMemoryRpcClient
from uprotocol.communication.upayload import UPayload
from uprotocol.v1.uattributes_pb2 import (
    UPayloadFormat,
)
from uprotocol.v1.uri_pb2 import UUri

from up_transport_zenoh.examples import common_uuri
from up_transport_zenoh.examples.common_uuri import create_method_uri, get_zenoh_default_config
from up_transport_zenoh.uptransportzenoh import UPTransportZenoh

source = UUri(authority_name="command", ue_id=18)
transport = UPTransportZenoh.new(get_zenoh_default_config(), source)


async def send_rpc_request_to_zenoh():
    # create uuri
    uuri = create_method_uri()
    # create UPayload
    data = "this is the request!"
    payload = UPayload(format=UPayloadFormat.UPAYLOAD_FORMAT_TEXT, data=bytes([ord(c) for c in data]))
    # invoke RPC method
    common_uuri.logging.debug(f"Send request to {uuri}")
    rpc_client = InMemoryRpcClient(transport)
    response_payload = await rpc_client.invoke_method(uuri, payload)
    common_uuri.logging.debug(f"Response payload {response_payload}")


if __name__ == '__main__':
    asyncio.run(send_rpc_request_to_zenoh())
