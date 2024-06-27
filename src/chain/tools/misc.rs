use std::error::Error;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Serialize, Deserialize, Clone)]
pub struct ToolOutput {
    pub command: String,
    pub input: Value,
    pub output: String,
}

pub fn deserialize_output(output: &str) -> Result<ToolOutput, Box<dyn Error>> {
    let output = output.replace("```yaml", "").replace("```", "");
    let model: ToolOutput = serde_yaml::from_str(&output).inspect_err(|e| {
        println!(
            "====DESERIALIZE ERROR====\nError: {:?}\nOutput:\n{}",
            e, output
        )
    })?;
    Ok(model)
}
