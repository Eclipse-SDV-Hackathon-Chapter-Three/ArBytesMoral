import asyncio
import threading

from uprotocol.client.usubscription.v3.inmemoryusubcriptionclient import InMemoryUSubscriptionClient
from uprotocol.communication.inmemoryrpcserver import InMemoryRpcServer
from uprotocol.communication.requesthandler import RequestHandler
from uprotocol.transport.ulistener import UListener
from uprotocol.communication.upayload import UPayload
from uprotocol.v1.uattributes_pb2 import UPayloadFormat
from uprotocol.v1.umessage_pb2 import UMessage
from uprotocol.v1.ustatus_pb2 import UStatus
from uprotocol.v1.uri_pb2 import UUri

import common_uuri
from up_transport_zenoh.uptransportzenoh import UPTransportZenoh

from agents import Agent, Runner, WebSearchTool, function_tool

source_rpc = UUri(authority_name="voice-command", ue_id=18)
transport_rpc = UPTransportZenoh.new(common_uuri.get_zenoh_config(), source_rpc)

source_sub = UUri(authority_name="subscriber", ue_id=9)
transport_sub = UPTransportZenoh.new(common_uuri.get_zenoh_default_config(), source_sub)

vehicle_data_storage = {}

class DataListener(UListener):
    def __init__(self, topic):
        self.topic = topic

    async def on_receive(self, msg: UMessage) -> None:
        common_uuri.logging.debug(msg.payload)
        payload = msg.payload
        data = payload.decode('utf-8') if payload else ""
        vehicle_data_storage[self.topic] = data
        print(vehicle_data_storage)

        return UStatus(message="Received event")

async def subscribe_to_zenoh_if_subscription_service_is_not_running(uuri, topic):
    status = await transport_sub.register_listener(uuri, DataListener(topic))
    common_uuri.logging.debug(f"Register Listener status  {status}")
    while True:
        await asyncio.sleep(1)

# LLM Tools
@function_tool
async def get_vehicle_data(type: str) -> str:
    """Fetch data from the vehicle.

    Args:
        type: the type of vehicle data to get. Available: outside.humidity, inside.temperature.
    """
    print("-----------------------------AI wants:-----------------------------" + type)
    print(vehicle_data_storage)
    if type == "outside.humidity":
        return vehicle_data_storage["outside.humidity"]
    elif type == "inside.temperature":
        return vehicle_data_storage["inside.temperature"]
    else:
        return "type not supported"

# LLM Agents
news_weather_agent = Agent(
    name="news_weather_agent",
    instructions="Give the current weather.",
    model="gpt-4o-mini",
    tools=[WebSearchTool()]
)

news_general_agent = Agent(
    name="news_general_agent",
    instructions="Give a brief general news overview of germany.",
    model="gpt-4o-mini",
    tools=[WebSearchTool()]
)

vehicle_data_agent = Agent(
    name="vehicle_data_agent",
    instructions="Handle vehicle-data related queries to get data. Available: outside.humidity, inside.temperature.",
    model="gpt-4o-mini",
    tools=[get_vehicle_data]
)

vehicle_command_agent = Agent(
    name="vehicle_command_agent",
    instructions="Handle vehicle-control related queries to set data trigger actions.",
    model="gpt-4o-mini"
)

supervisor_agent = Agent(
    name="supervisor_agent",
    instructions="""You are a supervisor agent. When given a request, determine if it is related to vehicle information, vehicle commands, news, other tasks.
    Dispatch the request to the correct sub-agent or tool and return the response.
    For car Data use vehicle_data_agent, this can provide: outside.humidity, inside.temperature.
    Use metric units or convert to metric units.
    Try to cheer up the person you are talking to.
    Modify the final respons so it is not longer than 2 sentences and usefull when spoken via tts.""",
    model="gpt-4o-mini",
     tools=[get_vehicle_data,
        # news_weather_agent.as_tool(
        #     tool_name="news_weather",
        #     tool_description="Give the current weather",
        # ),
        # news_general_agent.as_tool(
        #     tool_name="news_general",
        #     tool_description="Give a brief general news overview",
        # ),
        # vehicle_data_agent.as_tool(
        #     tool_name="vehicle_data",
        #     tool_description="Handle vehicle-data related queries to get data. Available: outside.humidity, inside.temperature.",
        # ),
        # vehicle_command_agent.as_tool(
        #     tool_name="vehicle_command",
        #     tool_description="Handle vehicle-control related queries to set data trigger actions",
        # ),
    ],
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
                agent_result = asyncio.run(Runner.run(supervisor_agent, voice_command))
                result.append(agent_result.final_output)
            
            thread = threading.Thread(target=run_agent)
            thread.start()
            thread.join()  # Wait for completion
            
            voice_answer = result[0]

            print("voice_answer" + voice_answer)
            
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
    uuri = common_uuri.create_method_uri()
    rpc_server = InMemoryRpcServer(transport_rpc)
    status = await rpc_server.register_request_handler(uuri, MyRequestHandler())
    common_uuri.logging.debug(f"Request Handler Register status {status}")

    while True:
        await asyncio.sleep(1)


async def main():
    uuri_8001 = UUri(authority_name="vehicledataaccessor", ue_id=0, ue_version_major=2, resource_id=0x8001)
    uuri_8002 = UUri(authority_name="vehicledataaccessor", ue_id=0, ue_version_major=2, resource_id=0x8002)
    
    # Prepare the coroutine calls
    task1 = subscribe_to_zenoh_if_subscription_service_is_not_running(uuri_8001, "inside.temperature")
    task2 = subscribe_to_zenoh_if_subscription_service_is_not_running(uuri_8002, "outside.humidity")
    task3 = register_rpc()
    
    # Run all three concurrently and wait until all finish
    await asyncio.gather(task1, task2, task3)

if __name__ == '__main__':
    asyncio.run(main())