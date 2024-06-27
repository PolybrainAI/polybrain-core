use async_trait::async_trait;
use llm_chain::tools::{Describe, Tool, ToolDescription, ToolError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct UserQuery {}

impl UserQuery {
    pub fn new() -> Self {
        UserQuery {}
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserQueryInput {
    pub question: String,
}

impl From<&str> for UserQueryInput {
    fn from(value: &str) -> Self {
        Self {
            question: value.into(),
        }
    }
}

impl From<String> for UserQueryInput {
    fn from(value: String) -> Self {
        Self { question: value }
    }
}

impl Describe for UserQueryInput {
    fn describe() -> llm_chain::tools::Format {
        vec![("question", "The question to ask the user").into()].into()
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserQueryOutput {
    pub result: String,
}

impl From<String> for UserQueryOutput {
    fn from(value: String) -> Self {
        Self { result: value }
    }
}

impl From<UserQueryOutput> for String {
    fn from(val: UserQueryOutput) -> Self {
        val.result
    }
}

impl Describe for UserQueryOutput {
    fn describe() -> llm_chain::tools::Format {
        vec![("answer", "The user's answer to the question").into()].into()
    }
}

#[derive(Debug, Error)]
pub enum UserQueryError {
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

impl ToolError for UserQueryError {}

#[async_trait]
impl Tool for UserQuery {
    type Input = UserQueryInput;

    type Output = UserQueryOutput;

    type Error = UserQueryError;

    async fn invoke_typed(&self, _: &Self::Input) -> Result<Self::Output, Self::Error> {
        let answer = "{{USER_INPUT}}";
        Ok(UserQueryOutput {
            result: answer.to_owned(),
        })
    }

    // fn construct_input(input: serde_yaml::Value) -> Result<Self::Input, Self::Error>{
    //     todo!()
    // }

    fn description(&self) -> ToolDescription {
        ToolDescription::new(
            "User Query",
            "Useful for when you need to ask the user a question. Ask one thing at a time.",
            "Use this to get information about the user's request.",
            UserQueryInput::describe(),
            UserQueryOutput::describe(),
        )
    }
}
