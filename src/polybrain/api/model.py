from pydantic import BaseModel

class SessionRequest(BaseModel):
    session_id: str

class WebsocketMessage(BaseModel):
    messageType: str
    body: dict