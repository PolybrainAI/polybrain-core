use async_trait::async_trait;
use llm_chain::tools::{Describe, Tool, ToolDescription, ToolError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct RecordThought {}

impl RecordThought {
    pub fn new() -> Self {
        RecordThought {}
    }
}

#[derive(Serialize, Deserialize)]
pub struct RecordThoughtInput {
    pub thought: String,
}

impl From<&str> for RecordThoughtInput {
    fn from(value: &str) -> Self {
        Self {
            thought: value.into(),
        }
    }
}

impl From<String> for RecordThoughtInput {
    fn from(value: String) -> Self {
        Self { thought: value }
    }
}

impl Describe for RecordThoughtInput {
    fn describe() -> llm_chain::tools::Format {
        println!("Converting into!!");
        vec![("thought", "The intermediate thought to record").into()].into()
    }
}

#[derive(Serialize, Deserialize)]
pub struct RecordThoughtOutput {
    none: (),
}

impl Describe for RecordThoughtOutput {
    fn describe() -> llm_chain::tools::Format {
        vec![].into()
    }
}

#[derive(Debug, Error)]
pub enum RecordThoughtError {
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

impl ToolError for RecordThoughtError {}

#[async_trait]
impl Tool for RecordThought {
    type Input = RecordThoughtInput;

    type Output = RecordThoughtOutput;

    type Error = RecordThoughtError;

    async fn invoke_typed(&self, _: &Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(RecordThoughtOutput { none: () })
    }

    // fn construct_input(input: serde_yaml::Value) -> Result<Self::Input, Self::Error>{
    //     todo!()
    // }

    fn description(&self) -> ToolDescription {
        ToolDescription::new(
            "Record Thought",
            "Records an intermediate thought",
            "Use this multiple times to track your thoughts",
            RecordThoughtInput::describe(),
            RecordThoughtOutput::describe(),
        )
    }
}
