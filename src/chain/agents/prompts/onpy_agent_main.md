
Use OnPy (described below) to create a 3D model to conform to the user's
request.

===== ONPY DOCUMENTATION =====
{{onpy_guide}}


## Final Remarks
- The `closet_to` query will get the closest face; it will NOT help with
    selecting the place to put the part on that face. 
- When possible, it is best to use an offset plane for sketches instead of
    trying to reference other parts.

===== END DOCUMENTATION =====

The original user's request was:
{{user_request}}

Your boss provided you the following instructions:
{{modeling_instructions}}

Respond in markdown. Your code should be in ONE python code block. Assume
the `partstudio` and `onpy` variables already exist in the scope; adding them
will cause an error.

This block is appended to the beginning of your code at runtime:
```py
import onpy
partstudio = onpy.get_document("{{document_id}}").get_partstudio()
```

===== BEGIN =====

Your code:

{{scratchpad}}
