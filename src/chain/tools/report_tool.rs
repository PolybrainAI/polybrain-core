use async_trait::async_trait;
use llm_chain::tools::{Describe, Tool, ToolDescription, ToolError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct Report {}

impl Report {
    pub fn new() -> Self {
        Report {}
    }
}

#[derive(Serialize, Deserialize)]
pub struct ReportInput {
    pub content: String,
}

impl From<&str> for ReportInput {
    fn from(value: &str) -> Self {
        Self {
            content: value.into(),
        }
    }
}

impl From<String> for ReportInput {
    fn from(value: String) -> Self {
        Self { content: value }
    }
}

impl Describe for ReportInput {
    fn describe() -> llm_chain::tools::Format {
        vec![("content", "The contents of the report").into()].into()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ReportOutput {
    pub none: (),
}

impl From<String> for ReportOutput {
    fn from(_: String) -> Self {
        Self { none: () }
    }
}

impl From<ReportOutput> for String {
    fn from(_: ReportOutput) -> Self {
        "None".to_owned()
    }
}

impl Describe for ReportOutput {
    fn describe() -> llm_chain::tools::Format {
        vec![("none", "This tool has no output").into()].into()
    }
}

#[derive(Debug, Error)]
pub enum ReportError {
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

impl ToolError for ReportError {}

#[async_trait]
impl Tool for Report {
    type Input = ReportInput;

    type Output = ReportOutput;

    type Error = ReportError;

    async fn invoke_typed(&self, _: &Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(ReportOutput { none: () })
    }

    // fn construct_input(input: serde_yaml::Value) -> Result<Self::Input, Self::Error>{
    //     todo!()
    // }

    fn description(&self) -> ToolDescription {
        ToolDescription::new(
            "Report",
            "Submits the final report",
            "Put the entire contents of the final report into this tool",
            ReportInput::describe(),
            ReportOutput::describe(),
        )
    }
}
