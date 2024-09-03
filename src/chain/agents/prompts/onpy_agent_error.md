Find the error and amend the code based on the provided error message. The
documentation for the OnPy module is provided below, along with some of
the parameters the original code author was attempting to conform to.

Use OnPy (described below) to create a 3D model to conform to the user's
request.

===== ONPY DOCUMENTATION =====
{{onpy_guide}}
===== END DOCUMENTATION =====

The original user's request was:
{{user_request}}

Fix the problem in the code below. Respond with a single, large markdown block. Error
messages are shown under each script.

The original code was:
```python
{{erroneous_code}}
```
FAILED! Console:
```
{{console_output}}
```

Add your code below, in ONE block. Assume the partstudio variable and onpy
import above are moved into this context; i.e., do not reimport onpy
or create a new document/partstudio.

More specifically, the following code is appended to the beginning of each
block at runtime.
```py
import onpy
partstudio = onpy.get_document("{{document_id}}").get_partstudio()
```

==== BEGIN ====

{{scratchpad}}
