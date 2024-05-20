from dataclasses import dataclass
import re

@dataclass
class TokenContainer:
    openai_token: str 
    onshape_token: str

def parse_python_code(response: str) -> str|None:
    """Parses the python code out of a response from an LLM. Assumes markdown 
    format.

    Args:
        response: The response from the LLM that may contain Python code

    Returns:
        The Python code as a string, or None if nothing was enclosed.
    """

    pattern = re.compile(r'```(?:python|py)\n(.*?)\n```', re.DOTALL)
    
    matches = re.findall(pattern, response)
    
    # If we find any match return the joined string else return None
    if matches:
        return '\n'.join(matches)
    else:
        return None


def unwrap[
    T
](object: T | None, message: str | None = None, default: T | None = None) -> T:
    """Takes the object out of an Option[T].

    Args:
        object: The object to unwrap
        message: An optional message to show on error
        default: An optional value to use instead of throwing an error

    Returns:
        The object of the value, if it is not None. Returns the default if the
        object is None and a default value is provided

    Raises
        TypeError if the object is None and no default value is provided.
    """

    if object is not None:
        return object
    else:
        if default is not None:
            return default
        else:
            raise TypeError(message if message else "Failed to unwrap")
