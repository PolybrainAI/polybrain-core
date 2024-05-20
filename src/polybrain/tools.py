"""

Tools for the LangChain agent are defined here

"""

from langchain.tools import tool
from typing import TYPE_CHECKING, Callable, Sequence

from polybrain.util import parse_python_code, unwrap


if TYPE_CHECKING:
    from polybrain.client import Client


class ToolContainer:
    """Contains all of the LangChain exposed tools. Allows the passing of a
    reference to the client."""

    def __init__(self, client: "Client") -> None:
        self.client = client

    @property
    def tools(self) -> Sequence:
        """A list of the available tools"""

        @tool
        def speak_tool(text: str) -> None:
            """Speak a message to the user. Let them know what you are doing.

            Args:
                text: The text to speak
            """

            self.client.send_output(text)

        @tool
        def get_input(question: str) -> str:
            """Gets the user's input from a question. Feel free to ask many questions.
            Asking questions between steps is encouraged.

            Args:
                question: The question to ask the user
            """
            self.client.send_output(question)
            return self.client.get_input()

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

            return self.client.interpreter.run_python_code(code)

        return [speak_tool, get_input, code_tool]
