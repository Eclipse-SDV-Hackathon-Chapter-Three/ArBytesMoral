import asyncio
import threading

from uprotocol.communication.inmemoryrpcserver import InMemoryRpcServer
from uprotocol.communication.requesthandler import RequestHandler
from uprotocol.communication.upayload import UPayload
from uprotocol.v1.uattributes_pb2 import UPayloadFormat
from uprotocol.v1.umessage_pb2 import UMessage
from uprotocol.v1.uri_pb2 import UUri

from up_transport_zenoh.examples import common_uuri
from up_transport_zenoh.examples.common_uuri import create_method_uri, get_zenoh_default_config
from up_transport_zenoh.uptransportzenoh import UPTransportZenoh

from agents import Agent, Runner, WebSearchTool

source = UUri(authority_name="voice-command", ue_id=18)
transport = UPTransportZenoh.new(get_zenoh_default_config(), source)

# LLM Agents
news_weather_agent = Agent(
    name="news_weather_agent",
    instructions="Give the current weather.",
    model="gpt-4o-mini",
    tools=[WebSearchTool()]
)

news_general_agent = Agent(
    name="news_weather_agent",
    instructions="Give a brief general news overview of germany.",
    model="gpt-4o-mini",
    tools=[WebSearchTool()]
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
    Dispatch the request to the correct sub-agent or tool and return the response. 
    Use metric units or convert to metric units.
    Try to cheer up the person you are talking to.
    Modify the final respons so it is not longer than 2 sentences and usefull when spoken via tts.""",
    model="gpt-4o-mini",
     tools=[
        news_weather_agent.as_tool(
            tool_name="news_weather",
            tool_description="Give the current weather",
        ),
        news_general_agent.as_tool(
            tool_name="news_general",
            tool_description="Give a brief general news overview",
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
    uuri = create_method_uri()
    rpc_server = InMemoryRpcServer(transport)
    status = await rpc_server.register_request_handler(uuri, MyRequestHandler())
    common_uuri.logging.debug(f"Request Handler Register status {status}")

    while True:
        await asyncio.sleep(1)


if __name__ == '__main__':
    asyncio.run(register_rpc())
