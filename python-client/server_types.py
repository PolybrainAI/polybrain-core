from pydantic import BaseModel


class SessionStartRequest(BaseModel):
    user_token: str

class SessionStartResponse(BaseModel):
    session_id: str

class UserPrompt(BaseModel):
    contents: str

class ApiCredentials(BaseModel):
    openai_token: str
    onshape_access_key: str
    onshape_secret_key: str


