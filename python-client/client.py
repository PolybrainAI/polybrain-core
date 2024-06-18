"""
A Python utility to test a client to the socket server
"""

import json
import socket
import time
import dotenv 
import os

from pydantic import BaseModel
import server_types

dotenv.load_dotenv()


class Client:

    def __init__(self) -> None:
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM) 

        try:
            self.sock.connect((os.environ["HOST_NAME"], int(os.environ["HOST_PORT"]) ))
        except socket.error as e:
            print(f"error: failed to connect to server: {e}")
            exit()
        else:
            print("info: made connection with server")


    def receive_message[T](self, type: type[T]) -> T:
        buffer = ""
        while True:
            data = self.sock.recv(4096).decode()
            buffer += data 
            if "\r\n" in buffer:
                if not buffer.endswith("\r\n"):
                    print("error: buffer contains end sequence, but not at end. a message was missed!")
                    buffer = buffer.split("\r\n")[0]
                break 

        print(f"debug: received message:\n{buffer}")
        return type(**json.loads(buffer))


    def send_message(self, message: BaseModel):
        payload = message.model_dump_json(indent=4)
        payload += "\r\n"
        print(f"debug: sending message: \n{payload}")
        self.sock.send(payload.encode())

    def run(self):

        # Complete auth handshake with server
        self.send_message(server_types.SessionStartRequest(
            onshape_document_id=os.environ["TEST_DOCUMENT_ID"],
            user_token=os.environ["TEST_USER_TOKEN"])
            )
        print("info: sent session start request")

        response = self.receive_message(server_types.SessionStartResponse)
        session_id = response.session_id
        print(f"info: got sessions start response with id: {session_id}")

        # Send initial request
        self.send_message(server_types.UserPromptInitial(contents="Make a table"))
        print(f"info: sent initial prompt")

        # Wait for inputs

        # NOTE: hard-coded for now; do some sort of callback thing in the future
        query = self.receive_message(server_types.UserInputQuery)
        print(f"info: got user query: '{query}'")

        self.send_message(server_types.UserInputResponse(response="yes!"))
        print("info: responded to user query")

        message = self.receive_message(server_types.ServerResponse)
        print(f"info: got server response: \n{message}")

        message = self.receive_message(server_types.ServerResponse)
        print(f"info: got server response: \n{message}")



if __name__ == "__main__":
    Client().run()