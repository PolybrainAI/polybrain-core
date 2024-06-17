import json
import socket
import time
import dotenv 
import os

from pydantic import BaseModel
import server_types

dotenv.load_dotenv()


def receive_message[T](socket: socket.socket, type: type[T]) -> T:
    buffer = ""
    while True:
        data = socket.recv(4096).decode()
        buffer += data 
        if "\r\n" in buffer:
            if not buffer.endswith("\r\n"):
                print("error: buffer contains end sequence, but not at end. a message was missed!")
                buffer = buffer.split("\r\n")[0]
            break 

    print(f"received message:\n{buffer}")
    return type(**json.loads(buffer))


def send_message(socket: socket.socket, message: BaseModel):
    payload = message.model_dump_json(indent=4)
    payload += "\r\n"
    socket.send(payload.encode())



s = socket.socket(socket.AF_INET, socket.SOCK_STREAM) 
try:
    s.connect((os.environ["HOST_NAME"], int(os.environ["HOST_PORT"]) ))
except socket.error as e:
    print(f"error: failed to connect to server: {e}")
    exit()

print("made connection")

time.sleep(5)
send_message(
    s,
    server_types.SessionStartRequest(user_token="placeholder")
)
print("Send session start request")

response = receive_message(s, server_types.SessionStartResponse)
print("Got session start response")
