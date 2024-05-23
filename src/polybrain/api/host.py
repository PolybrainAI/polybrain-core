"""

Hosts the polybrain API

"""

from typing import Union
from fastapi import FastAPI, WebSocket
from fastapi.middleware.httpsredirect import HTTPSRedirectMiddleware
from fastapi.middleware.cors import CORSMiddleware
from loguru import logger

import polybrain.api.model as model
from polybrain.api.session import Session, sessions

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origin_regex=r"https://cad.onshape.com/*",
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# app.add_middleware(HTTPSRedirectMiddleware)

@app.get("/")
async def read_root():
    return {"message": "ALIVE"}


@app.get("/session/create/{document_id}")
async def session_start(document_id: str):
    """Registers a new session
    
    Args:
        document_id: The OnShape document id of the session
    """

    new_session = None  

    for session in sessions:
        if session.document_id == document_id:
            new_session = session

    if new_session is None:
        new_session = Session(document_id)
        logger.info(f"Created new session {new_session.session_id}")
    return {"session_id": new_session.session_id}

@app.get("/session/end/{session_id}")
async def session_end(session_id: str):
    """Shuts down an existing session
    
    Args:
        session_id: Ends a session
    """

    session = Session.find_session(session_id)
    del session

    logger.info(f"Closed session {session_id}")
    return {"message": f"deleted session {session_id}"}



@app.websocket("/ws/{session_id}")
async def send_data(websocket:WebSocket, session_id: str):
    logger.info(f"Incoming ws connection with session {session_id}")
    await websocket.accept()
    session = Session.find_session(session_id)
    session._websocket = websocket

    await session.run_ws_loop()


