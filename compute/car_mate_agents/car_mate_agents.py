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
import os
from openai import OpenAI  # Use synchronous client

from uprotocol.communication.inmemoryrpcserver import InMemoryRpcServer
from uprotocol.communication.requesthandler import RequestHandler
from uprotocol.communication.upayload import UPayload
from uprotocol.v1.uattributes_pb2 import UPayloadFormat
from uprotocol.v1.umessage_pb2 import UMessage
from uprotocol.v1.uri_pb2 import UUri

from up_transport_zenoh.examples import common_uuri
from up_transport_zenoh.examples.common_uuri import create_method_uri, get_zenoh_default_config
from up_transport_zenoh.uptransportzenoh import UPTransportZenoh

# Initialize synchronous OpenAI client
openai_client = OpenAI(
    api_key=os.environ.get("OPENAI_API_KEY")
)

source = UUri(authority_name="command", ue_id=18)
transport = UPTransportZenoh.new(get_zenoh_default_config(), source)


class MyRequestHandler(RequestHandler):
    def handle_request(self, msg: UMessage) -> UPayload:
        common_uuri.logging.debug("Request Received by Service Request Handler")
        attributes = msg.attributes
        payload = msg.payload
        source = attributes.source
        sink = attributes.sink
        
        # Extract the request text
        request_text = payload.decode('utf-8') if payload else ""
        common_uuri.logging.debug(f"Receive '{request_text}' from {source} to {sink}")
        
        try:
            # Call OpenAI API synchronously
            response = openai_client.chat.completions.create(
                model="gpt-4o-mini",  # or gpt-4o, gpt-3.5-turbo
                messages=[
                    {"role": "system", "content": "You are a helpful assistant."},
                    {"role": "user", "content": request_text}
                ]
            )
            
            # Extract the response text
            response_text = response.choices[0].message.content
            common_uuri.logging.debug(f"OpenAI response: {response_text}")
            
            # Create response payload
            return UPayload(
                data=response_text.encode('utf-8'),
                format=UPayloadFormat.UPAYLOAD_FORMAT_TEXT
            )
            
        except Exception as e:
            common_uuri.logging.error(f"Error calling OpenAI: {e}")
            error_message = f"Error: {str(e)}"
            return UPayload(
                data=error_message.encode('utf-8'),
                format=UPayloadFormat.UPAYLOAD_FORMAT_TEXT
            )


async def register_rpc():
    uuri = create_method_uri()
    rpc_server = InMemoryRpcServer(transport)
    status = await rpc_server.register_request_handler(uuri, MyRequestHandler())
    common_uuri.logging.debug(f"Request Handler Register status {status}")

    while True:
        await asyncio.sleep(1)  # Use asyncio.sleep instead of time.sleep


if __name__ == '__main__':
    asyncio.run(register_rpc())
