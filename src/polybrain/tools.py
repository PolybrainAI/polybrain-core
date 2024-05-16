"""

Tools for the LangChain agent are defined here

"""

from langchain_community.tools.human.tool import HumanInputRun

def get_input() -> str:
    print("Insert your text. Enter 'q' or press Ctrl-D (or Ctrl-Z on Windows) to end.")
    contents = []
    while True:
        try:
            line = input()
        except EOFError:
            break
        if line == "q":
            break
        contents.append(line)
    return "\n".join(contents)

human_tool = HumanInputRun(input_func=get_input)
tools = [human_tool]