
use std::{error::Error, io::{self, Write}, pin::Pin};

use futures::Future;
use llm_chain::{executor, parameters, prompt::{Conversation, ChatMessage}};
use llm_chain::prompt;
use llm_chain_openai;
use llm_chain::options;
use llm_chain_openai::chatgpt::Model;

use crate::{chain::util::trim_assistant_prefix, server::types::{ServerResponse, ServerResponseType}};

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
Respond quickly, and try not to ask too many questions.
If you deny a user's request, tell them exactly why.

If the request is reasonable, end your final message with \"Begin!\" You 
MUST respond with \"Begin!\" eventually.

{{conversation_history}}
";


fn get_user_input() -> String {
    let mut input = String::new();
    print!("Please enter your input: ");
    io::stdout().flush().unwrap(); // Ensure the prompt is displayed before reading input
    io::stdin().read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string() // Remove trailing newline and return the input
}


pub struct PessimistAgent{
    messages: Conversation,
    openai_key: String
}

impl PessimistAgent {

    pub fn new(openai_key: String) -> PessimistAgent {
        PessimistAgent{
            messages: Conversation::new(),
            openai_key: openai_key
        }
    }

    fn build_conversation_history(&self) -> String {
        let message_history = &self.messages.to_string();
        println!("Message history is:\n{message_history}");
        return message_history.to_owned();
    }

    pub async fn run<'a, I, O>(&mut self, initial_message: &str, get_input: &I, send_output: &O) 
    -> Result<(), Box<dyn std::error::Error>> where
        I: Fn(String) -> Pin<Box<dyn Future<Output = Result<String, Box<dyn Error>>> + Send + 'a>>
            + Send
            + 'a,
        O: Fn(ServerResponse) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>>
            + Send
            + 'a
        {

        let mut agent_response: String = "".to_owned();
        self.messages.add_message(ChatMessage::user(initial_message.to_owned()));

        let opts = options!{
            Model: Model::Other("gpt-4o".to_string()),
            ApiKey: self.openai_key.clone(),
            StopSequence: vec!["\n".to_string(), "User:".to_string()]
        };
        let exec = executor!(chatgpt, opts)?;
        
        while !agent_response.contains("Begin!") {
    
            let parameters = parameters!{
                "conversation_history" => self.build_conversation_history()
            };

            let res = prompt!(system: PESSIMIST_PROMPT)
            .run(&parameters, &exec) // ...and run it
            .await?;
    
            let r = res.to_immediate().await?.as_content().to_text().clone();
            agent_response = trim_assistant_prefix(&r).trim().to_string();
    
            println!("agent: {}", agent_response);

            if !agent_response.contains("Begin!"){
                println!("not end");
                let user_input = get_input(agent_response.clone()).await?;
                self.messages.add_message(ChatMessage::user(user_input))
            }
            else{
                send_output(ServerResponse{
                    response_type: ServerResponseType::Info,
                    content: agent_response.replace("Begin!", "")
                }).await?;
            }

        }

        println!("Exiting pessimist chain");

        Ok(())

    } 
}
