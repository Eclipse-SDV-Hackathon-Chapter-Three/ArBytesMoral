import asyncio
import threading
from agents import Agent, Runner
from agents.tool import Tool


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

# LLM Agents
news_weather_agent = Agent(
    name="news_weather_agent",
    instructions="Give the current weather.",
    model="gpt-4o-mini"
)

vehicle_data_agent = Agent(
    name="vehicle_data_agent",
    instructions="Handle vehicle-data related queries to get data.",
    model="gpt-4o-mini"
)

vehicle_command_agent = Agent(
    name="vehicle_command_agent",
    instructions="Handle vehicle-control related queries to set data trigger actions.",
    model="gpt-4o-mini"
)

supervisor_agent = Agent(
    name="supervisor_agent",
    instructions="""You are a supervisor agent. When given a request, determine if it is related to vehicle information, vehicle commands, news, other tasks.
    Dispatch the request to the correct sub-agent or tool and return the response.""",
    model="gpt-4o-mini",
     tools=[
        news_weather_agent.as_tool(
            tool_name="news_weather",
            tool_description="Give the current weather",
        ),
        vehicle_data_agent.as_tool(
            tool_name="vehicle_datat",
            tool_description="Handle vehicle-data related queries to get data",
        ),
        vehicle_command_agent.as_tool(
            tool_name="vehicle_command",
            tool_description="Handle vehicle-control related queries to set data trigger actions",
        ),
    ],
)

# def get_vehicle_status(vehicle_id: str) -> str:
#     # Access vehicle database or API to fetch status
#     return f"Vehicle {vehicle_id} is operational with all systems normal."

# def execute_voice_command(command: str) -> str:
#     # Perform voice command action or simulate execution
#     return f"Executed voice command: {command}"

# vehicle_data_agent.tools = [get_vehicle_status]
# vehicle_command_agent.tools = [execute_voice_command]


# # Define tools wrapping sub-agents
# def news_weather_agent_tool(input_text: str) -> str:
#     result = Runner.run_sync(news_weather_agent, input_text)
#     return result.final_output

# def vehicle_data_agent_tool(input_text: str) -> str:
#     result = Runner.run_sync(vehicle_data_agent, input_text)
#     return result.final_output

# def vehicle_command_agent_tool(input_text: str) -> str:
#     result = Runner.run_sync(vehicle_command_agent, input_text)
#     return result.final_output

# # Define your tools properly
# news_weather_tool = Tool(
#     name="news_weather_agent",
#     func=news_weather_agent_tool,
#     description="Handles weather news queries"
# )

# vehicle_data_tool = Tool(
#     name="vehicle_data_agent",
#     func=vehicle_data_agent_tool,
#     description="Handles vehicle-data related queries"
# )

# vehicle_command_tool = Tool(
#     name="vehicle_command_agent",
#     func=vehicle_command_agent_tool,
#     description="Handles vehicle-control related queries"
# )

# # Assign Tool instances to supervisor_agent.tools
# supervisor_agent.tools = [
#     news_weather_tool,
#     vehicle_data_tool,
#     vehicle_command_tool,
# ]

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
