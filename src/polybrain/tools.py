"""

Tools for the LangChain agent are defined here

"""

from langchain_community.tools.human.tool import HumanInputRun
from langchain.tools import tool

from polybrain.code_runner.run import run_python_code
from polybrain.util import parse_python_code, unwrap

from openai import OpenAI
import playsound
import os

client = OpenAI()

@tool
def speak_tool(text: str) -> None:
    """Speak a message to the user. Let them know what you are doing.
    
    Args:
        text: The text to speak
    """

    with client.audio.speech.with_streaming_response.create(
        model="tts-1",
        voice="alloy",
        input=text,
    ) as response:
        response.stream_to_file("speech.mp3")
        playsound.playsound("speech.mp3")
        os.remove("speech.mp3")



def get_input() -> str:
    return input("You: ")

human_tool = HumanInputRun(input_func=get_input, prompt_func=lambda x: print(f"Polybrain: {x}"))
tools = [human_tool]