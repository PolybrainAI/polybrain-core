use futures::Future;
use llm_chain::options;
use llm_chain::prompt;
use llm_chain::{executor, parameters};
use llm_chain_openai;
use llm_chain_openai::chatgpt::Model;
use std::pin::Pin;
use std::process::{Command, Output};
use thiserror::Error;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::server::background::BackgroundClient;
use crate::server::types::ApiCredentials;
use crate::util::PolybrainError;

use super::Agent;

const MAX_ITER: usize = 10;
const MAX_ITER_ERR: usize = 10;
const ONPY_AGENT_PROMPT: &str = r###"

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

"###;

const ONPY_ERROR_PROMPT: &str = r###"
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

"###;

const INPUT_PRASE_PROMPT: &str = r###"\

The following response is from a user when asked if they want changes to their
model.

The response was:
{{user_response}}

If the user wants changes, respond "Yes"
If the user does NOT want changes, response "No"
Respond in only ONE word.

"###;

#[derive(Error, Debug)]
pub enum CodeError {
    #[error("code output is malformed: {0}")]
    BadFormat(String),

    #[error("execution error: {0}")]
    ExecutionError(String),

    #[error("an internal, unexpected error occurred while parsing Python: {0}")]
    Internal(String),
}

pub struct OnPyAgent<'b> {
    credentials: &'b ApiCredentials,
    client: &'b mut BackgroundClient,
    report: String,
    original_request: String,
    onshape_document: String,
}

impl<'b> OnPyAgent<'b> {
    pub fn new(
        credentials: &'b ApiCredentials,
        client: &'b mut BackgroundClient,
        report: String,
        original_request: String,
        onshape_document: String,
    ) -> Self {
        Self {
            credentials,
            client,
            report,
            original_request,
            onshape_document,
        }
    }

    async fn load_onpy_guide() -> String {
        let client = reqwest::Client::new();
        client
            .get("https://raw.githubusercontent.com/kyle-tennison/onpy/main/guide.md")
            .send()
            .await
            .expect("Error in requesting OnPy guide")
            .error_for_status()
            .expect("OnPy GitHub guide request failed")
            .text()
            .await
            .expect("Failed to convert OnPy github request to text")
    }

    pub fn format_code_output(output: &str) -> Result<String, CodeError> {
        let mut output = output.replace("```python", "```").replace("```py", "```");

        let num_boundaries = output
            .as_bytes()
            .windows(3)
            .filter(|&w| w == "```".as_bytes())
            .count();

        if num_boundaries % 2 != 0 {
            output.push_str("\n```");
        }

        let code = output
            .split("```")
            .enumerate()
            .filter(|pair| pair.0 % 2 != 0)
            .map(|pair| pair.1.trim())
            .collect::<Vec<&str>>()
            .join("\n");

        Ok(code)
    }

    pub async fn execute_block(code: &str, onshape_document: &str) -> Result<String, CodeError> {
        let code = format!(
            concat!(
                "import onpy\n",
                "partstudio = onpy.get_document('{doc_id}').get_partstudio()\n",
                "partstudio.wipe()\n",
                "{code}"
            ),
            doc_id = onshape_document,
            code = code
        );

        println!(
            concat!(
                "==== EXECUTING CODE ====",
                "# Executing the following:\n```py\n{}\n```\n",
                "========================"
            ),
            code
        );

        // Create a temporary file
        let mut file = File::create("temp_script.py")
            .await
            .expect("Failed to create tmp python file");

        // Write the code to the file
        file.write_all(code.as_bytes())
            .await
            .expect("Failed to write to tmp python file");

        // Execute the Python script
        let output: Output = Command::new("python")
            .arg("temp_script.py")
            .output()
            .map_err(|e| {
                CodeError::ExecutionError(format!("Failed to execute Python script: {:?}", e))
            })?;

        // Handle the output
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(CodeError::ExecutionError(stderr))
        }
    }

    pub async fn handle_error(
        &mut self,
        erroneous_code: String,
        error_output: String,
    ) -> Result<(String, String), CodeError> {
        let opts = options! {
            Model: Model::Other("gpt-4o".to_string()),
            // Model: Model::Gpt35Turbo,
            ApiKey: self.credentials.openai_token.clone(),
            StopSequence: vec!["```\n\n".to_string(), "Cell Output".to_string(), "Console Output".to_string()]
        };
        let exec = executor!(chatgpt, opts).map_err(|err| CodeError::Internal(err.to_string()))?;
        let onpy_guide = Self::load_onpy_guide().await;
        let mut scratchpad = String::new();

        for _ in 0..MAX_ITER_ERR {
            let parameters = parameters!(
                "onpy_guide" => &onpy_guide,
                "user_request" => &self.original_request,
                "erroneous_code" => &erroneous_code,
                "document_id" => &self.onshape_document,
                "console_output" => &error_output,
                "scratchpad" => &scratchpad,
            );
            let prompt_full = prompt!(ONPY_ERROR_PROMPT)
                .format(&parameters)
                .map_err(|err| CodeError::Internal(err.to_string()))?;

            println!(
                concat!(
                    "==== FULL ERROR PROMPT ====\n",
                    "{}\n",
                    "==========================="
                ),
                prompt_full
            );

            let mut code_output = prompt!(ONPY_ERROR_PROMPT)
                .run(&parameters, &exec)
                .await
                .expect("Failed to run handle_error agent")
                .to_immediate()
                .await
                .expect("Failed to convert LLM response to immediate")
                .primary_textual_output()
                .expect("No LLM output");

            println!(
                concat!(
                    "==== LLM CODE RESPONSE ====\n",
                    "(Error Agent)\n",
                    "{}\n",
                    "==========================="
                ),
                code_output
            );

            scratchpad.push_str(&code_output);

            code_output = Self::format_code_output(&code_output)
                .map_err(|err| CodeError::BadFormat(err.to_string()))?;

            match Self::execute_block(&code_output, &self.onshape_document).await {
                Ok(console) => {
                    return Ok((code_output, console));
                }
                Err(err) => scratchpad.push_str(&format!("Cell Error:\n```\n{}\n```", err)),
            };
        }

        eprintln!("Max error retries exceeded!");
        todo!()
    }
}

impl<'b> Agent for OnPyAgent<'b> {
    type InvocationResponse = ();

    async fn client<'a>(&'a mut self) -> &'a mut BackgroundClient {
        self.client
    }

    async fn invoke(&mut self) -> Result<(), PolybrainError> {
        // Setup primary executor
        let opts = options! {
            Model: Model::Other("gpt-4o".to_string()),
            // Model: Model::Gpt35Turbo,
            ApiKey: self.credentials.openai_token.clone(),
            StopSequence: vec!["```\n\n".to_string(), "Cell Output".to_string()]
        };
        let main_exec = executor!(chatgpt, opts)
            .map_err(|err| PolybrainError::InternalError(err.to_string()))?;

        // Setup secondary executor
        let opts = options! {
            Model: Model::Gpt35Turbo,
            ApiKey: self.credentials.openai_token.clone(),
            StopSequence: vec!["\n".to_string()]
        };
        let secondary_exec = executor!(chatgpt, opts)
            .map_err(|err| PolybrainError::InternalError(err.to_string()))?;

        let onpy_guide = Self::load_onpy_guide().await;
        let mut scratchpad = String::new();

        for _ in 0..MAX_ITER {
            // Generate code
            println!("generating code...");
            let mut code_output = prompt!(ONPY_AGENT_PROMPT)
                .run(
                    &parameters!(
                        "onpy_guide" => &onpy_guide,
                        "user_request" => &self.original_request,
                        "modeling_instructions" => &self.report,
                        "document_id" => &self.onshape_document,
                        "scratchpad" => &scratchpad
                    ),
                    &main_exec,
                )
                .await
                .map_err(|err| {
                    PolybrainError::InternalError("Error calling OnPy Agent LLM".to_owned())
                })?
                .to_immediate()
                .await
                .map_err(|err| {
                    PolybrainError::InternalError(
                        "Error converting LLM response to immediate".to_owned(),
                    )
                })?
                .primary_textual_output()
                .expect("No LLM output");

            println!(
                concat!(
                    "==== LLM CODE RESPONSE ====\n",
                    "(Main Agent)\n",
                    "{}\n",
                    "==========================="
                ),
                code_output
            );

            // Run code
            code_output = Self::format_code_output(&code_output)
                .map_err(|err| PolybrainError::CodeError(err))?;
            match Self::execute_block(&code_output, &self.onshape_document).await {
                Ok(output) => {
                    scratchpad.push_str(&code_output);
                    scratchpad.push_str(&format!("Cell Output:\n```\n{}\n```", output))
                }
                Err(CodeError::ExecutionError(tb)) => {
                    let (new_code, new_output) = &self
                        .handle_error(code_output.clone(), tb)
                        .await
                        .inspect_err(|err| {
                            eprintln!("Failed to recover from erroneous response: {err}")
                        })
                        .map_err(|err| PolybrainError::CodeError(err))?;

                    scratchpad.push_str(new_code);
                    scratchpad.push_str(&format!("Cell Output:\n```\n{}\n```", new_output));
                }
                Err(_) => {
                    panic!("Unhandled error occurred during code execution")
                }
            };

            // Validate with user
            let user_input = self
                .query_input("Does this model meet your specifications?".to_owned())
                .await?;

            let llm_interpretation = prompt!(INPUT_PRASE_PROMPT)
                .run(
                    &parameters!(
                        "user_response" => &user_input
                    ),
                    &secondary_exec,
                )
                .await
                .map_err(|_| {
                    PolybrainError::InternalError("Error calling OnPy Agent LLM".to_owned())
                })?
                .to_immediate()
                .await
                .map_err(|_| {
                    PolybrainError::InternalError(
                        "Error converting LLM response to immediate".to_owned(),
                    )
                })?
                .primary_textual_output()
                .expect("No LLM output");

            let is_acceptance = llm_interpretation.to_ascii_lowercase().contains("yes");

            if is_acceptance {
                println!("The user accepted the model");
                break;
            } else {
                let scratchpad_addition = format!(concat!(
                    "The user was asked asked if they are satisfied with the model above. ",
                    "They responded saying: \n\"{}\"\n",
                    "Make adjustments to the previous model such that it conforms with the user's requested change",
                ), user_input);

                scratchpad.push_str(&scratchpad_addition);
            }
        }

        Ok(())
    }
}
