import asyncio
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


source = UUri(authority_name="voice-command", ue_id=18)
transport = UPTransportZenoh.new(get_zenoh_default_config(), source)

# Initialize the OpenAI Agent
agent = Agent(
    name="Assistant",
    instructions="You are a helpful assistant for vehicle-related queries. Provide clear, concise, and accurate responses.",
    model="gpt-4o-mini"
)


class MyRequestHandler(RequestHandler):
    def handle_request(self, msg: UMessage) -> UPayload:
        payload = msg.payload
        
        voice_command = payload.decode('utf-8') if payload else ""
        common_uuri.logging.debug(f"Received '{voice_command}'")
        
        try:
            # Run the agent in a separate thread to avoid event loop conflict
            result = []
            
            def run_agent():
                # Create a new event loop for this thread
                agent_result = asyncio.run(Runner.run(agent, voice_command))
                result.append(agent_result.final_output)
            
            thread = threading.Thread(target=run_agent)
            thread.start()
            thread.join()  # Wait for completion
            
            voice_answer = result[0]
            
            return UPayload(
                data=voice_answer.encode('utf-8'),
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
