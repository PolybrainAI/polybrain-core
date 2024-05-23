"""

Tools for the LangChain agent are defined here

"""

from langchain.tools import tool
from typing import TYPE_CHECKING, Callable, Sequence

from polybrain.core.util import parse_python_code, unwrap


if TYPE_CHECKING:
    from polybrain.core.client import Client


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
            """Share an intermediate thought with the user. You are encouraged
            to use the speak_tool to share intermediate thoughts with the user. Do not use
            this tool to answer the prompt; any final response needs to be provided in the
            Final Answer field.

            Args:
                text: The text to speak
            """

            self.client.send_output(text)

        @tool 
        def view_code(unused = None) -> str:
            """View the existing code that builds the model
            
            Args:
                unused: Can be anything, not used
            """

            return self.client.interpreter.code_storage
            
        @tool 
        def clear_code(unused = None) -> None:
            """Clears the existing code. You will need to redefine an entire
            new model with the add_code tool if you use this.
            
            Args:
                unused: Can be anything, not used
            """
            
            self.client.interpreter.clear_code_session()


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
        def add_code(code: str) -> str:
            """Add code to a python file and execute it. You can view
            existing code with the view_code tool. 

            Args:
                code: Properly formatted python code

            Returns:
                A string containing the STDOUT and STDERR and a copy of the
                total code
            """

            if "import onpy" in code or "partstudio =" in code:
                return "ERROR: OnPy is already imported and a partstudio object is already defined. Do not include these in the script"

            if "```" in code:
                code = unwrap(parse_python_code(code), default=code.replace("```", ""))

            return self.client.interpreter.run_python_code(code)

        return [speak_tool, get_input, add_code, clear_code, view_code]
