use std::{error::Error, pin::Pin};

use futures::Future;
use llm_chain::options;
use llm_chain::prompt;
use llm_chain::{
    executor, parameters,
    prompt::{ChatMessage, Conversation},
};
use llm_chain_openai;
use llm_chain_openai::chatgpt::Model;

use crate::server::background::BackgroundClient;
use crate::server::types::ApiCredentials;
use crate::util::PolybrainError;
use crate::{
    chain::util::trim_assistant_prefix,
    server::types::{ServerResponse, ServerResponseType},
};

use super::Agent;

const PESSIMIST_PROMPT: &str = r###"

You are a friendly assistant who works for Polybrain, a 3D modeling company. 

Your main job is to help the user request a model that is within Polybrain's
modeling capabilities.

Greet the client. They should provide a 3D modeling request; if they don't,
ask them what you want Polybrain to make. Once they have provided a model,
use your existing knowledge of 3D CAD platforms to determine if their
model can be created within Polybrain's capabilities. When in doubt, 
let the user do what they want.

Polybrain (a parametric modeler) has the ability to:
- Create 2D sketches with primitive lines, arcs, rectangles, and circles
ate extrusions (addition and subtraction)- Cre
- Create lofts (this is very big!)

This means that Polybrain, unlike other CAD software, is unable to:
- Create revolve, sweep, and chamfer features
- Create complex 2D sketches
- Create angled, complicated faces

The following is your conversation with the user. 
If you deny a user's request, tell them exactly why.
Respond quickly, and try not to ask too many questions. Your responses
should rarely be longer than 2 sentences.

If the request is reasonable, end your final message with \"Begin!\" You 
MUST respond with \"Begin!\" eventually. 

YOU MUST SEND "Begin!" TO ALLOW THE USER TO PROCEED. IF YOU PROMPT NO QUESTION
YOU MUST SEND "Begin!" IT IS PARAMOUNT!

{{conversation_history}}
"###;

const SUMMARIZER_PROMPT: &str = "
Consider the following conversation between a user and an assistant. Summarize
the model that the user ended up requesting in the end. Your summary should
be no longer than four sentences, but it should include all the details
available in this conversation. DO NOT include any new features that weren't
requested by the user.

The conversation is:
{{conversation_history}}
";

pub struct PessimistAgent<'b> {
    messages: Conversation,
    credentials: &'b ApiCredentials,
    client: &'b mut BackgroundClient,
}

impl<'b> PessimistAgent<'b> {
    pub fn new(
        credentials: &'b ApiCredentials,
        client: &'b mut BackgroundClient,
        initial_message: String,
    ) -> Self {
        let messages = Conversation::new().with_user(initial_message);

        Self {
            messages,
            credentials,
            client,
        }
    }

    fn build_conversation_history(&self) -> String {
        let message_history = &self.messages.to_string();
        message_history.to_owned()
    }

    fn build_prompt(&self) -> String {
        PESSIMIST_PROMPT.replace(
            "{{conversation_history}}",
            &self.build_conversation_history(),
        )
    }
}

impl<'b> Agent for PessimistAgent<'b> {
    type InvocationResponse = String;

    async fn client<'a>(&'a mut self) -> &'a mut BackgroundClient {
        self.client
    }

    async fn invoke(&mut self) -> Result<String, PolybrainError> {
        let mut agent_response: String = "".to_owned();

        let opts = options! {
            Model: Model::Other("gpt-4o".to_string()),
            // Model: Model::Gpt35Turbo,
            ApiKey: self.credentials.openai_token.clone(),
            StopSequence: vec!["User:".to_string()]
        };
        let exec = executor!(chatgpt, opts).map_err(|_| {
            PolybrainError::InternalError("Error calling pessimist executor".to_owned())
        })?;

        while !agent_response.contains("Begin!") {
            let parameters = parameters! {};

            let res = prompt!(system: &self.build_prompt())
                .run(&parameters, &exec) // ...and run it
                .await
                .map_err(|_| {
                    PolybrainError::InternalError("Error calling Pessimist LLM".to_owned())
                })?
                .to_immediate()
                .await
                .map_err(|_| {
                    PolybrainError::InternalError(
                        "Error converting LLM response to immediate".to_owned(),
                    )
                })?
                .as_content()
                .to_text();

            agent_response = trim_assistant_prefix(&res).trim().to_string();

            println!("Pessimist: {}", agent_response);

            if agent_response.contains("Begin!") {
                self.send_message(ServerResponse {
                    response_type: ServerResponseType::Info,
                    content: agent_response.replace("Begin!", ""),
                })
                .await?;
            } else {
                self.messages
                    .add_message(ChatMessage::assistant(agent_response.replace("\n", " ")));
                let user_input = self.query_input(agent_response.clone()).await?;
                self.messages.add_message(ChatMessage::user(user_input))
            }
        }

        // Summarize what the user decided on
        let summary = prompt!(SUMMARIZER_PROMPT)
            .run(
                &parameters!("conversation_history" => self.build_conversation_history()),
                &exec,
            )
            .await
            .map_err(|_| PolybrainError::InternalError("Error calling Pessimist LLM".to_owned()))?
            .to_immediate()
            .await
            .map_err(|_| {
                PolybrainError::InternalError(
                    "Error converting LLM response to immediate".to_owned(),
                )
            })?
            .as_content()
            .to_text();

        let summary = trim_assistant_prefix(&summary).to_owned();

        println!("Summarized prompt as: {}", summary);

        Ok(summary)
    }
}
