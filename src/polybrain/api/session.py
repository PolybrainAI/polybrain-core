"""

Manages active sessions 

"""

import json
import time
from uuid import uuid4
from fastapi import WebSocket
from loguru import logger
import asyncio

from polybrain.core.client import Client
import polybrain.api.model as model

sessions: list["Session"] = []

class Session:

    def __init__(self, document_id: str) -> None:

        # Register new session
        self.session_id = str(uuid4())
        sessions.append(self)

        self.document_id = document_id

        self.client = Client()
        self._websocket: WebSocket|None = None

    @property
    def websocket(self) -> WebSocket:
        """A reference to the websocket"""
        if self._websocket is None:
            raise RuntimeError("Websocket is unbound")
        return self._websocket

    @staticmethod
    def find_session(session_id: str) -> "Session":
        """Finds a session in the list of active sessions"""

        for session in sessions:
            if session.session_id == session_id:
                return session
            
        raise KeyError(f"No session with id {session_id} exists")
    
    async def toggle_microphone(self, on: bool = False):
        """Sends message to turn microphone on or off"""

        logger.info("Toggling microphone " + ("ON" if on else "OFF"))

        await self.websocket.send_json(model.WebsocketMessage(
            messageType = "MICROPHONE",
            body={"STATUS": "ON" if on else "OFF"}
        ).model_dump())

    async def toggle_loading(self, on: bool = False):
        """Sends message to turn the loading bar on """
        logger.info("Toggling loading " + ("ON" if on else "OFF"))

        await self.websocket.send_text(model.WebsocketMessage(
            messageType = "LOADING",
            body={"STATUS": "ON" if on else "OFF"}
        ).model_dump_json())

    async def toggle_speaking(self, on: bool = False):
        """Sends message to signal response talking"""
        logger.info("Toggling speaking " + ("ON" if on else "OFF"))

        await self.websocket.send_json(model.WebsocketMessage(
            messageType = "SPEAKING",
            body={"STATUS": "ON" if on else "OFF"}
        ).model_dump())
    
    async def run_ws_loop(self):

        while True:

            incoming_raw = await self.websocket.receive_text()

            logger.debug(f"incoming message: {incoming_raw}")

            incoming = model.WebsocketMessage(**json.loads(incoming_raw))

            if incoming.messageType != "BEGIN":
                logger.warning(f"Waiting for BEGIN. Got other message: {incoming}")
                continue
            else:
                logger.info("Beginning client interaction")

            break

        await self.toggle_microphone(on=True)
        await asyncio.sleep(0.001) # break to allow message to send
        user_input = self.client.get_input()
        await self.toggle_microphone(on=False)


        await self.toggle_loading(on=True)
        await asyncio.sleep(0.001) # break to allow message to send
        response = self.client.agent_executor.invoke(
            {
                "input": user_input,
                "chat_history": self.client.memory.chat_memory.messages,
                "onpy_guide": self.client.load_onpy_guide(),
            }
        )
        self.client.memory.chat_memory.add_ai_message(response["output"])
        await self.toggle_loading(on=False)

        await self.toggle_speaking(on=True)
        await asyncio.sleep(0.001) # break to allow message to send
        response_text = response["output"]
        self.client.audio.speak_text(response_text)
        await self.toggle_speaking(on=False)


    def __del__(self) -> None:
        sessions.remove(self)