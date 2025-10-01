import zenoh

# Create a Config object
config = zenoh.Config()
config.from_file(path="zenoh.json")

# config.mode = 'client'
# config.peer = 'tcp/127.0.0.1:7447'  # Replace with your router's address

with zenoh.open(config) as session:
    publisher = session.declare_publisher('demo/example/hello')
    publisher.put('Hello World!')
