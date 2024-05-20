"""

Tools for the LangChain agent are defined here

"""

from typing import Sequence
from langchain_community.tools.human.tool import HumanInputRun
from langchain.tools import tool

from polybrain.code_runner.run import run_python_code
from polybrain.util import parse_python_code, unwrap
from polybrain.audio import speak_text, get_audio_input

@tool
def speak_tool(text: str) -> None:
    """Speak a message to the user. Let them know what you are doing.
    
    Args:
        text: The text to speak
    """

    speak_text(text)


@tool
def get_input(question: str) -> str:
    """Gets the user's input from a question. Feel free to ask many questions.
    Asking questions between steps is encouraged.
    
    Args:
        question: The question to ask the user
    """
    speak_text(question)
    return get_audio_input()

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



tools: Sequence = [get_input, code_tool, speak_tool]