"""

The entry point to the polybrain modeler

"""

from textwrap import dedent
import dotenv
from langchain_openai import ChatOpenAI
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.output_parsers import StrOutputParser
from langchain.agents import create_tool_calling_agent, initialize_agent, AgentType, create_json_chat_agent, AgentExecutor
from langchain_community.tools.human.tool import HumanInputRun
from polybrain.tools import tools
from langchain.memory import ConversationBufferMemory


def load_prompt_str() -> str:
    """Loads the master prompt as a string"""

    with open("resources/onpy_guide.md", 'r') as f:
        guide = f.read()

    prompt_text = dedent("""

    You are a LLM powered mechanical engineer created by Polybrain--an AI company
    from San Francisco, California. Your name is Jacob, and your job is to create
    3D models using OnShape--a popular CAD platform.
                    
    As a large language model, you are unable to directly interact with the OnShape
    software. Instead, you will need to interact with OnShape through OnPy,
    a Python API to OnShape.
                    
    Due to the nature of a Python API, you are limited in what you can create in 
    OnPy. If a user requests a model that is too complex, it is better to reject
    their request rather than trying and failing.
                    
    The following document is a guide to use OnPy. It is your responsibility to 
    weigh the complexity of the request and to see if it is possible with the
    features made available through OnPy.
                    
    ======= DOCUMENT BEGIN =======
    GUIDE_DOCUMENT
    ======= DOCUMENT END =======

    OnPy's limited tools mean that all geometries created will be minimal and
    simple. For this reason, do not worry about the physical feasibility of the
    models you create; simply produce a CAD model with OnPY that will comply with 
    the user's request. OnPy does not support materials, so do NOT ask the
    user for materials.

    Again, you should avoid:
    - Referencing materials
    - Alluding to the manufacture or physical of the model


    Use OnPy to create the model requested by the user. Use markdown to delineate
    where you are writing python versus comments to the user.
    """.replace("GUIDE_DOCUMENT", guide))

    return prompt_text


def entry():

    dotenv.load_dotenv()

    # --- Init llm ---
    llm = ChatOpenAI(model="gpt-4o", temperature=0.2, verbose=True)
    output_parser = StrOutputParser()


    prompt = ChatPromptTemplate.from_messages(
        [
            ("system", load_prompt_str()),
            ("placeholder", "{chat_history}"),
            ("human", "{input}"),
            ("placeholder", "{agent_scratchpad}"),
        ]
    )


    memory = ConversationBufferMemory()
    agent = create_tool_calling_agent(llm, tools, prompt)
    agent_executor = AgentExecutor(agent=agent, tools=tools, verbose=False, memory=memory)

    while True:


        user_input = input("You: ")

        if user_input.lower() in ("exit", "q"):
            break

        memory.chat_memory.add_user_message(user_input)

        response = agent_executor.invoke({"input": user_input})

        memory.chat_memory.add_ai_message(response["output"])

        print("Polybrain:", response["output"])

