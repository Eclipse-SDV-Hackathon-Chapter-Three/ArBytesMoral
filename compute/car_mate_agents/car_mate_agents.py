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
import threading
from agents import Agent, Runner

from uprotocol.communication.inmemoryrpcserver import InMemoryRpcServer
from uprotocol.communication.requesthandler import RequestHandler
from uprotocol.communication.upayload import UPayload
from uprotocol.v1.uattributes_pb2 import UPayloadFormat
from uprotocol.v1.umessage_pb2 import UMessage
from uprotocol.v1.uri_pb2 import UUri

from up_transport_zenoh.examples import common_uuri
from up_transport_zenoh.examples.common_uuri import create_method_uri, get_zenoh_default_config
from up_transport_zenoh.uptransportzenoh import UPTransportZenoh

source = UUri(authority_name="command", ue_id=18)
transport = UPTransportZenoh.new(get_zenoh_default_config(), source)

# Initialize the OpenAI Agent
agent = Agent(
    name="Assistant",
    instructions="You are a helpful assistant for vehicle-related queries. Provide clear, concise, and accurate responses.",
    model="gpt-4o-mini"
)


class MyRequestHandler(RequestHandler):
    def handle_request(self, msg: UMessage) -> UPayload:
        common_uuri.logging.debug("Request Received by Service Request Handler")
        attributes = msg.attributes
        payload = msg.payload
        
        request_text = payload.decode('utf-8') if payload else ""
        common_uuri.logging.debug(f"Receive '{request_text}'")
        
        try:
            # Run the agent in a separate thread to avoid event loop conflict
            result = []
            
            def run_agent():
                # Create a new event loop for this thread
                agent_result = asyncio.run(Runner.run(agent, request_text))
                result.append(agent_result.final_output)
            
            thread = threading.Thread(target=run_agent)
            thread.start()
            thread.join()  # Wait for completion
            
            response_text = result[0]
            common_uuri.logging.debug(f"Agent response: {response_text}")
            
            return UPayload(
                data=response_text.encode('utf-8'),
                format=UPayloadFormat.UPAYLOAD_FORMAT_TEXT
            )
            
        except Exception as e:
            common_uuri.logging.error(f"Error calling OpenAI Agent: {e}")
            return UPayload(
                data=f"Error: {str(e)}".encode('utf-8'),
                format=UPayloadFormat.UPAYLOAD_FORMAT_TEXT
            )


async def register_rpc():
    uuri = create_method_uri()
    rpc_server = InMemoryRpcServer(transport)
    status = await rpc_server.register_request_handler(uuri, MyRequestHandler())
    common_uuri.logging.debug(f"Request Handler Register status {status}")

    while True:
        await asyncio.sleep(1)


if __name__ == '__main__':
    asyncio.run(register_rpc())
