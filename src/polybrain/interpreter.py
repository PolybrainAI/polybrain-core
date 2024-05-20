from io import StringIO
from contextlib import redirect_stdout
import json
import sys
import io
import traceback
import onpy


class DualStream(io.StringIO):
    """Streams output to STDOUT while recording stream"""

    def __init__(self, original_stream):
        super().__init__()
        self.original_stream = original_stream

    def write(self, s):
        super().write(s)
        self.original_stream.write(s)

    def flush(self):
        super().flush()
        self.original_stream.flush()


class Interpreter:
    """A class used to execute OnPy code"""

    def __init__(self, document_id: str) -> None:
        """
        Args:
            document_id: The id of the document to target
        """

        # Load onshape environment
        self.partstudio = onpy.get_document(document_id).get_partstudio()
        self.code_storage = "partstudio.wipe()"

    def run_python_code(self, code: str) -> str:
        """Runs python code and returns output

        Args:
            code: The python code to execute

        Returns:
            A JSON string with stdout and stderr information
        """

        # Create DualStream objects to capture and stream stdout and stderr
        original_stdout = sys.stdout
        original_stderr = sys.stderr

        stdout_capture = DualStream(original_stdout)
        stderr_capture = DualStream(original_stderr)

        # Redirect stdout and stderr
        sys.stdout = stdout_capture
        sys.stderr = stderr_capture

        # Load local variables
        partstudio = self.partstudio

        try:
            # Execute the code
            exec(f"{self.code_storage}\n{code}", globals(), locals())

            # Retrieve stdout content
            stdout_content = stdout_capture.getvalue()

            # Update code storage
            self.code_storage = f"{self.code_storage}\n{code}"

            # Return output
            return json.dumps({"stdout": stdout_content, "stderr": None}, indent=4)
        except Exception as e:
            # Capture the exception traceback
            traceback.print_exc()

            # Retrieve stdout and stderr content
            stdout_content = stdout_capture.getvalue()
            stderr_content = stderr_capture.getvalue()

            # Return output
            return json.dumps(
                {"stdout": stdout_content, "stderr": stderr_content}, indent=4
            )
        finally:
            # Reset stdout and stderr to their original state
            sys.stdout = original_stdout
            sys.stderr = original_stderr

            # Close the StringIO objects
            stdout_capture.close()
            stderr_capture.close()

    def clear_code_session(self) -> None:
        """Creates a new code session by wiping the code history and partstudio"""

        self.partstudio.wipe()
        self.code_storage = ""
