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

from uprotocol.communication.inmemoryrpcclient import InMemoryRpcClient
from uprotocol.communication.inmemoryrpcserver import InMemoryRpcServer
from uprotocol.communication.requesthandler import RequestHandler
from uprotocol.communication.upayload import UPayload
from uprotocol.v1.uattributes_pb2 import (
    UPayloadFormat,
)
from uprotocol.v1.umessage_pb2 import UMessage
from uprotocol.v1.uri_pb2 import UUri

from up_transport_zenoh.examples import common_uuri
from up_transport_zenoh.examples.common_uuri import create_method_uri, get_zenoh_default_config
from up_transport_zenoh.uptransportzenoh import UPTransportZenoh


from openai import OpenAI

client = OpenAI(
  api_key=""
)

command_urri = UUri(authority_name="voicecommand", ue_id=18)
command_transport = UPTransportZenoh.new(get_zenoh_default_config(), command_urri)

answer_source = UUri(authority_name="voiceanswer", ue_id=18)
answer_transport = UPTransportZenoh.new(get_zenoh_default_config(), answer_source)

class MyRequestHandler(RequestHandler):
    async def handle_request(self, msg: UMessage) -> UPayload:
        common_uuri.logging.debug("Request Received by Service Request Handler")
        attributes = msg.attributes
        payload = msg.payload
        source = attributes.source
        sink = attributes.sink
        common_uuri.logging.debug(f"Receive {payload} from {source} to {sink}")

        await self.process_command(payload)

        payload = UPayload(data="command received", format=UPayloadFormat.UPAYLOAD_FORMAT_TEXT)
        return payload

    async def rpc_request_answer(self, answer):
        # create uuri
        uuri = create_method_uri()
        # create UPayload
        payload = UPayload(format=UPayloadFormat.UPAYLOAD_FORMAT_TEXT, data=bytes([ord(c) for c in answer]))
        # invoke RPC method
        common_uuri.logging.debug(f"Send request to {uuri}")
        rpc_client = InMemoryRpcClient(answer_transport)
        response_payload = await rpc_client.invoke_method(uuri, payload)
        return response_payload
   
    async def process_command(self, command):
        answer = client.responses.create(
        model="gpt-4o-mini",
        input=command,
        store=True,
        )

        output_text = await answer.output_text()
        return await self.rpc_request_answer(output_text)

async def register_command_rpc():
    uuri = create_method_uri()
    rpc_server = InMemoryRpcServer(command_transport)
    status = await rpc_server.register_request_handler(uuri, MyRequestHandler())
    common_uuri.logging.debug(f"Request Handler Register status {status}")

    while True:
        time.sleep(1)

if __name__ == '__main__':
    asyncio.run(register_command_rpc())
