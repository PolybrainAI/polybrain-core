from enum import Enum
from pydantic import BaseModel


class SessionStartRequest(BaseModel):
    user_token: str

class SessionStartResponse(BaseModel):
    session_id: str

class UserPromptInitial(BaseModel):
    contents: str
class UserInputQuery(BaseModel):
    query: str
class UserInputResponse(BaseModel):
    response: str
class ServerResponse(BaseModel):
    response_type: str
    content: str

class ApiCredentials(BaseModel):
    openai_token: str
    onshape_access_key: str
    onshape_secret_key: str
