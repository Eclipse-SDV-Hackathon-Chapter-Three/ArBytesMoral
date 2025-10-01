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
import time

from uprotocol.communication.upayload import UPayload
from uprotocol.transport.builder.umessagebuilder import UMessageBuilder
from uprotocol.v1.uri_pb2 import UUri

import common_uuri
from up_transport_zenoh.uptransportzenoh import UPTransportZenoh

source = UUri(authority_name="vehicledataaccessor", ue_id=0, ue_version_major=2)
publisher = UPTransportZenoh.new(common_uuri.get_zenoh_default_config(), source)


async def publish_to_zenoh():
    # create uuri
    source.resource_id = 0x8001
    builder = UMessageBuilder.publish(source)
    payload = UPayload.pack(UUri())
    umessage = builder.build_from_upayload(payload)
    status = await publisher.send(umessage)
    common_uuri.logging.debug(f"Publish status {status}")
    time.sleep(3)


if __name__ == '__main__':
    asyncio.run(publish_to_zenoh())