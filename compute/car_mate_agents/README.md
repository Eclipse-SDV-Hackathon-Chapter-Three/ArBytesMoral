build: docker build -t car_mate_agents:latest .
run: docker run --rm -it -e OPENAI_API_KEY="$OPENAI_API_KEY" --net=host car_mate_agents
