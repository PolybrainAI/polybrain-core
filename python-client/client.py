"""
A Python utility to test a client to the socket server
"""

import json
import socket
import time
import dotenv
import os

import asyncio
from websockets.server import serve
from websockets.client import connect

from pydantic import BaseModel
from websockets import connect, WebSocketClientProtocol
import server_types

dotenv.load_dotenv()


class Client:

    def __init__(self):
        self.sock: WebSocketClientProtocol

    @classmethod
    async def new(cls) -> "Client":
        url = f"ws://{os.environ['HOST_NAME']}:{os.environ['HOST_PORT']}"
        print(f"info: connecting to websocket @ {url}")

        self = cls()

        try:
            self.sock = await connect(url)
        except socket.error as e:
            print(f"error: failed to connect to server: {e}")
            exit()
        else:
            print("info: made connection with server")

        return self

    async def receive_message[T](self, type: type[T]) -> T:
        buffer = await self.sock.recv()

        print(f"debug: received message:\n{buffer}")
        return type(**json.loads(buffer))

    async def send_message(self, message: BaseModel):
        payload = message.model_dump_json(indent=4)
        print(f"debug: sending message: \n{payload}")
        await self.sock.send(payload)

    async def run(self):

        # Complete auth handshake with server
        await self.send_message(
            server_types.SessionStartRequest(
                onshape_document_id=os.environ["TEST_DOCUMENT_ID"],
                user_token=os.environ["TEST_USER_TOKEN"],
            )
        )
        print("info: sent session start request")

        response = await self.receive_message(server_types.SessionStartResponse)
        session_id = response.session_id
        print(f"info: got sessions start response with id: {session_id}")

        # Send initial request
        await self.send_message(server_types.UserPromptInitial(contents="Make a table"))
        print(f"info: sent initial prompt")

        # Wait for inputs

        # NOTE: hard-coded for now; do some sort of callback thing in the future
        query = await self.receive_message(server_types.ServerResponse)
        print(f"info: got user query: '{query}'")

        await self.send_message(server_types.UserQueryResponse(response="yes!"))
        print("info: responded to user query")

        message = await self.receive_message(server_types.ServerResponse)
        print(f"info: got server response: \n{message}")

        message = await self.receive_message(server_types.ServerResponse)
        print(f"info: got server response: \n{message}")


async def main():
    client = await Client.new()
    await client.run()


if __name__ == "__main__":
    asyncio.run(main())
