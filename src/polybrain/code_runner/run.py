from io import StringIO
from contextlib import redirect_stdout
import json
import sys
import io
import traceback

# Load onshape environment
import onpy 
partstudio = onpy.get_document("d69e6ca6abae839540c3da27").get_partstudio()
partstudio.wipe()

class DualStream(io.StringIO):
    def __init__(self, original_stream):
        super().__init__()
        self.original_stream = original_stream
    
    def write(self, s):
        super().write(s)
        self.original_stream.write(s)
    
    def flush(self):
        super().flush()
        self.original_stream.flush()


code_storage = "partstudio.wipe()"

def run_python_code(code):
    global code_storage

    # Create DualStream objects to capture and stream stdout and stderr
    original_stdout = sys.stdout
    original_stderr = sys.stderr
    
    stdout_capture = DualStream(original_stdout)
    stderr_capture = DualStream(original_stderr)
    
    # Redirect stdout and stderr
    sys.stdout = stdout_capture
    sys.stderr = stderr_capture

    try:
        # Execute the code
        exec(f"{code_storage}\n{code}", globals(), locals())
        # Retrieve stdout content
        stdout_content = stdout_capture.getvalue()
        code_storage = f"{code_storage}\n{code}"
        return json.dumps({"stdout": stdout_content, "stderr": None}, indent=4)
    except Exception as e:
        # Capture the exception traceback
        traceback.print_exc()
        # Retrieve stdout and stderr content
        stdout_content = stdout_capture.getvalue()
        stderr_content = stderr_capture.getvalue()
        return json.dumps({"stdout": stdout_content, "stderr": stderr_content}, indent=4)
    finally:
        # Reset stdout and stderr to their original state
        sys.stdout = original_stdout
        sys.stderr = original_stderr
        # Close the StringIO objects
        stdout_capture.close()
        stderr_capture.close()


def clear_code_session():
    global code_storage
    
    partstudio.wipe()
    code_storage = ""