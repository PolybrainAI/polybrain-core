use llm_chain::options;
use llm_chain::prompt;
use llm_chain::{executor, parameters};
use llm_chain_openai;
use llm_chain_openai::chatgpt::Model;
use std::error::Error;
use std::process::{Command, Output};
use thiserror::Error;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

const ONPY_AGENT_PROMPT: &str = r###"

Use OnPy (described below) to create a 3D model to conform to the user's
request.

===== ONPY DOCUMENTATION =====
{{onpy_guide}}
===== END DOCUMENTATION =====

The original user's request was:
{{user_request}}

Your boss provided you the following instructions:
{{modeling_instructions}}

Respond in markdown. Code in python blocks are executed and the
console log is shown underneath.

===== BEGIN =====

First, importing the required module
```py
import onpy
partstudio = onpy.get_document("{{document_id}}").get_partstudio()
```


"###;

#[derive(Error, Debug)]
pub enum CodeError {
    #[error("code output is malformed: {0}")]
    BadFormat(String),

    #[error("execution error: {0}")]
    ExecutionError(String),
}

pub struct OnPyAgent<'b> {
    report: String,
    openai_key: &'b String,
    original_request: String,
    onshape_document: String,
}

impl<'b> OnPyAgent<'b> {
    pub fn new(
        openai_key: &'b String,
        report: String,
        original_request: String,
        onshape_document: String,
    ) -> OnPyAgent {
        OnPyAgent {
            openai_key,
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
        let output = output.replace("```python", "```").replace("```py", "```");

        let num_boundaries = output
            .as_bytes()
            .windows(3)
            .filter(|&w| w == "```".as_bytes())
            .count();

        if num_boundaries % 2 != 0 {
            return Err(CodeError::BadFormat(
                "Python is not properly contained within ``` boundaries.".to_owned(),
            ));
        }

        let code = output
            .split("```")
            .into_iter()
            .enumerate()
            .filter(|pair| pair.0 % 2 != 0)
            .map(|pair| pair.1.trim())
            .collect::<Vec<&str>>()
            .join("\n");

        Ok(code)
    }

    pub async fn execute_block(
        code: &str,
        onshape_document: &str,
    ) -> Result<String, Box<dyn Error>> {
        let code = format!(
            concat!(
                "import onpy\n",
                "partstudio = onpy.get_document('{doc_id}').get_partstudio()\n",
                "{code}"
            ),
            doc_id = onshape_document,
            code = code
        );

        println!(
            concat!(
                "==== EXECUTING CODE ====",
                "# Executing the following:\n```py\n{}\n```",
                "========================"
            ),
            code
        );

        // Create a temporary file
        let mut file = File::create("temp_script.py").await?;

        // Write the code to the file
        file.write(code.as_bytes()).await?;

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
            Err(Box::new(CodeError::ExecutionError(stderr)))
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let opts = options! {
            // Model: Model::Other("gpt-4o".to_string()),
            Model: Model::Gpt35Turbo,
            ApiKey: self.openai_key.clone()
        };
        let exec = executor!(chatgpt, opts)?;
        let onpy_guide = Self::load_onpy_guide().await;

        let mut code_output = prompt!(ONPY_AGENT_PROMPT)
            .run(
                &parameters!(
                    "onpy_guide" => onpy_guide,
                    "user_request" => &self.original_request,
                    "modeling_instructions" => &self.report,
                    "document_id" => &self.onshape_document
                ),
                &exec,
            )
            .await?
            .to_immediate()
            .await?
            .primary_textual_output()
            .expect("No LLM output");

        println!(
            concat!(
                "==== LLM CODE RESPONSE ====\n",
                "{}\n",
                "==========================="
            ),
            code_output
        );

        code_output = Self::format_code_output(&code_output)?;

        Self::execute_block(&code_output, &self.onshape_document).await?;

        Ok(())
    }
}
