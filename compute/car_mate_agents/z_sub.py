import zenoh
import time

config = zenoh.Config()
config.from_file(path="zenoh.json")
# config.mode = 'client'
# config.peer = 'tcp/127.0.0.1:7447'  # Adjust as needed

def listener(sample):
    print(f"{sample.key_expr} => {sample.payload.to_string()}")

with zenoh.open(config) as session:
    subscriber = session.declare_subscriber('**', listener)
    time.sleep(60)
