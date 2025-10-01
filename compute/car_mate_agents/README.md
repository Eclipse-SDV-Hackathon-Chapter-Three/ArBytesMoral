# CarMateAgents
##  build
`docker build -t car_mate_agents:latest .`

## run
`docker run --rm -it -e OPENAI_API_KEY="$OPENAI_API_KEY" --net=host car_mate_agents`

# General

The **CarMateAgents** acts as an interface between the **CarMateIO** to a LLM and the **Vehicle Data Accessor**. It consists of an supervisor agent that controls different agents using them as tools. For example the generic openai api, an online weather service or vehicle data provided via **Vehicle Data Accessor**.

# Communication via uProtocol

A small RPC server that listens for text voice-commands over uProtocol (transported by Zenoh) and answers using a supervisor LLM agent that can dispatch to sub-agents (weather, news, vehicle data, vehicle commands). Responses are returned as plain UTF-8 text (good for TTS)

source = UUri(authority_name="voice-command", ue_id=18)
transport = UPTransportZenoh.new(get_zenoh_default_config(), source)

Creates a Zenoh-backed uProtocol transport identity for this service.

# Agents

Several Agent instances are defined (news_weather_agent, news_general_agent, vehicle_data_agent, vehicle_command_agent).

A supervisor_agent routes an incoming request to the right tool/agent and trims the final answer to ≤2 sentences.

