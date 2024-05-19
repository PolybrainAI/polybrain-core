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
    return input("\nQuestion: ")

human_tool = HumanInputRun(
    input_func=get_input, 
    prompt_func=lambda x: print(f"Polybrain: {x}"), 
    description="Use this tool when you need information about the model to create. Do NOT ask how to do something.",
    handle_validation_error=True)

@tool
def code_tool(code: str) -> str:
    """Runs Python code and returns the STDOUT and STDERR. Assumes that a 
    variable named `partstudio` exists, that is an OnPy partstudio. Run
    this tool multiple times until errors are fixed.
    
    Args:
        code: Properly formatted python code

    Returns:
        A string containing the STDOUT and STDERR
    """

    if "import onpy" in code or "partstudio =" in code:
        return "ERROR: OnPy is already imported and a partstudio object is already defined. Do not include these in the script"

    if "```" in code:
        code = unwrap(parse_python_code(code), default=code.replace("```", ""))

    return run_python_code(code)



tools = [human_tool, code_tool, speak_tool]