"""

Client class to polybrain

"""

import json
from loguru import logger
from pathlib import Path
import dotenv
from langchain_openai import ChatOpenAI
from langchain_core.prompts import ChatPromptTemplate
from langchain.agents import AgentExecutor, create_react_agent
from langchain.memory import ConversationBufferMemory

from polybrain.util import TokenContainer, parse_python_code
from polybrain.tools import tools
from polybrain.audio import get_audio_input, speak_text
from polybrain.interpreter import Interpreter

class Client:

    SETTINGS_PATH = Path("../../polybrain_settings.json")

    def __init__(self, cheap_mode: bool = False) -> None:
        self.api_keys = self.resolve_tokens()
        self.settings = self.load_settings()

        model = self.settings["model_cheap"] if cheap_mode else self.settings["model_main"]

        self.llm = ChatOpenAI(model=model, temperature=self.settings["temperature"])

        agent = create_react_agent(self.llm, tools, self.load_prompt()) 
        self.agent_executor = AgentExecutor(agent=agent, tools=tools, verbose=True, handle_parsing_errors=True) # type: ignore
        self.memory = ConversationBufferMemory()


    @staticmethod
    def resolve_tokens() -> TokenContainer:
        """Gets the required tokens from the .env file
        
        Returns:
            The OpenAI API Key
        """

        env_filepath = dotenv.find_dotenv()

        if env_filepath:
            env_file = Path(env_filepath)
            logger.debug(f"Found .env file at {env_file.absolute()}")
        else:
            logger.warning("No .env file was found, creating one.")
            env_file = Path.joinpath(Path.cwd(), "../../.env")
            env_file.open("w").close()

        openai_api_key = dotenv.get_key(env_file, "OPENAI_API_KEY")
        onshape_api_key = dotenv.get_key(env_file, "ONSHAPE_API_KEY")

        if openai_api_key is None:
            logger.warning(
                "\n\nThere is no OpenAI API key in your .env file.\n"
                "Navigate to https://platform.openai.com/api-keys to get your API key.\n"
                "You can paste your key below, or add it to your .env file manually."
                )
            
            key_accepted = False

            while not key_accepted:
                api_key_input = input("> ").strip()
                
                if len(api_key_input) == 51:
                    key_accepted = True 
                else:
                    logger.warning("Your OpenAI API key should be 51 characters long. "
                                   "Try again, or manually add to the .env file")

            
            with env_file.open("a") as f:
                f.write(f"\nOPENAI_API_KEY =\"{api_key_input}\"")

            openai_api_key = api_key_input



        if onshape_api_key is None:
            logger.warning(
                "\n\nThere is no OnShape API key in your .env file.\n"
                "Navigate to https://dev-portal.onshape.com/keys to get your API key.\n"
                "You can paste your key below, or add it to your .env file manually."
                )
            
            key_accepted = False

            while not key_accepted:
                api_key_input = input("> ").strip()
                
                if len(api_key_input) == 51:
                    key_accepted = True 
                else:
                    logger.warning("Your OpenAI API key should be 51 characters long. "
                                   "Try again, or manually add to the .env file")

            
            with env_file.open("a") as f:
                f.write(f"\nONSHAPE_API_KEY =\"{api_key_input}\"")

            onshape_api_key = api_key_input

        return TokenContainer(openai_api_key, onshape_api_key)
    
    @classmethod
    def load_settings(cls) -> dict:
        """Loads settings JSON
        
        Returns:
            A dictionary containing the items defined in the settings JSON
        """

        with cls.SETTINGS_PATH.open("r") as f:
            return json.load(f)
        
    @classmethod
    def load_prompt(cls) -> ChatPromptTemplate:
        """Loads the LLM prompt
        
        Returns:
            The ChatPromptTemplate constructed by assets/prompt.md
        """

        prompt_path = Path("assets/prompt.md")

        with prompt_path.open("r") as f:
            prompt_template = f.read()

        return ChatPromptTemplate.from_template(prompt_template)
    
    @staticmethod
    def load_onpy_guide() -> str:
        """Loads the OnPy guide as a string"""

        # TODO: Download a copy from the OnPy repo

        onpy_guide = Path("assets/onpy_guide.md")

        with onpy_guide.open("r") as f:
            return f.read()
    
    def get_input(self) -> str:
        """Gets the user's input, by audio or text, depending on the configuration.
        
        Returns:
            The user input as a string
        """

        if self.settings["voice_mode"]:
            logger.info("Start speaking...")
            try:
                return get_audio_input()
            except KeyboardInterrupt:
                logger.info("goodbye")
                exit(0)
        else:
            logger.info("You:")
            return input("> ")
        
    def send_output(self, output: str) -> None:
        """Sends the LLM output, with optional audio depending on configuration.
        
        Args:
            output: The output of the agent
        """

        if self.settings["voice_mode"]:
            speak_text(output)

        logger.info("Jacob:")
        print(f"> {output}")

    
    def run(self) -> None:
        """Runs the polybrain LLM loop"""

        interpreter = Interpreter(self.settings["onshape_document_id"])

        while True:

            user_input = self.get_input()

            if user_input.lower() in ("exit", "q"):
                break

            response = self.agent_executor.invoke({
                "input": user_input, 
                "chat_history": self.memory.chat_memory.messages,
                "onpy_guide": self.load_onpy_guide()
                })

            self.memory.chat_memory.add_ai_message(response["output"])

            # sometimes LLM responds with code as final answer
            maybe_python = parse_python_code(response["output"])
            while maybe_python is not None:
                logger.debug(
                    "\n ---- Generated Python -----\n"
                    f"{maybe_python}"
                    "\n ---- ---------------- -----\n"
                    )

                output = interpreter.run_python_code(maybe_python)

                response = response = self.agent_executor.invoke(
                {"input": f"SYSTEM: {output}", "chat_history": self.memory.chat_memory.messages},
                )
                maybe_python = parse_python_code(response["output"])



            response_text = response["output"]
            speak_text(response_text)


            print("\nPolybrain:", response_text)
            print("\n")
            

        




        
            
            




