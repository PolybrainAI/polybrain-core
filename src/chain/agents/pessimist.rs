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

use crate::{
    chain::util::trim_assistant_prefix,
    server::types::{ServerResponse, ServerResponseType},
};

const PESSIMIST_PROMPT: &str = "

You are a friendly assistant who works for Polybrain, a 3D modeling company. 

Your main job is to help the user request a model that is within Polybrain's
modeling capabilities.

Greet the client. They should provide a 3D modeling request; if they don't,
ask them what you want Polybrain to make. Once they have provided a model,
use your existing knowledge of 3D CAD platforms to determine if their
model can be created within Polybrain's capabilities. When in doubt, 
error on the safe side--rejecting potentially complex models.

Polybrain (a parametric modeler) has the ability to:
- Create 2D sketches with primitive lines, arcs, rectangles, and circles
- Create extrusions (addition and subtraction)
- Create lofts

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

{{conversation_history}}
";

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
    openai_key: &'b String,
}

impl<'b> PessimistAgent<'b> {
    pub fn new(openai_key: &'b String) -> PessimistAgent {
        PessimistAgent {
            messages: Conversation::new(),
            openai_key: openai_key,
        }
    }

    fn build_conversation_history(&self) -> String {
        let message_history = &self.messages.to_string();
        return message_history.to_owned();
    }

    fn build_prompt(&self) -> String {
        let prompt = PESSIMIST_PROMPT.replace(
            "{{conversation_history}}",
            &self.build_conversation_history(),
        );
        prompt
    }

    pub async fn run<'a, I, O>(
        &mut self,
        initial_message: &str,
        get_input: &I,
        send_output: &O,
    ) -> Result<String, Box<dyn std::error::Error>>
    where
        I: Fn(String) -> Pin<Box<dyn Future<Output = Result<String, Box<dyn Error>>> + Send + 'a>>
            + Send
            + 'a,
        O: Fn(
                ServerResponse,
            ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>>
            + Send
            + 'a,
    {
        let mut agent_response: String = "".to_owned();
        self.messages
            .add_message(ChatMessage::user(initial_message.to_owned()));

        let opts = options! {
            Model: Model::Other("gpt-4o".to_string()),
            ApiKey: self.openai_key.clone(),
            StopSequence: vec!["\n".to_string(), "User:".to_string()]
        };
        let exec = executor!(chatgpt, opts)?;

        while !agent_response.contains("Begin!") {
            let parameters = parameters! {};

            let res = prompt!(system: &self.build_prompt())
                .run(&parameters, &exec) // ...and run it
                .await?;

            let r = res.to_immediate().await?.as_content().to_text().clone();
            agent_response = trim_assistant_prefix(&r).trim().to_string();

            println!("Pessimist: {}", agent_response);

            if agent_response.contains("Begin!") {
                send_output(ServerResponse {
                    response_type: ServerResponseType::Info,
                    content: agent_response.replace("Begin!", ""),
                })
                .await?;
            } else {
                self.messages
                    .add_message(ChatMessage::assistant(agent_response.replace("\n", " ")));
                let user_input = get_input(agent_response.clone()).await?;
                self.messages.add_message(ChatMessage::user(user_input))
            }
        }

        // Summarize what the user decided on
        let summary = prompt!(SUMMARIZER_PROMPT)
            .run(
                &parameters!("conversation_history" => self.build_conversation_history()),
                &exec,
            )
            .await?
            .to_immediate()
            .await?
            .as_content()
            .to_text();

        let summary = trim_assistant_prefix(&summary).to_owned();

        println!("Summarized prompt as: {}", summary);

        Ok(summary)
    }
}
