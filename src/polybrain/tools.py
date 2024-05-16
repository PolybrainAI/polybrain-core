"""

Tools for the LangChain agent are defined here

"""

from langchain_community.tools.human.tool import HumanInputRun

def get_input() -> str:
    return input("You: ")

human_tool = HumanInputRun(input_func=get_input, prompt_func=lambda x: print(f"Polybrain: {x}"))
tools = [human_tool]