use eyre::Result;
use serde::{Deserialize, Serialize};
use std::env;

const OPENAI_API_TOKEN: &str = "OPENAI_API_TOKEN";
const COMPLETIONS_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "lowercase", deserialize = "lowercase"))]
enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Serialize, Clone)]
enum Model {
    #[serde(rename(serialize = "gpt-3.5-turbo"))]
    Gpt35Turbo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MessageContent {
    role: Role,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletion {
    model: Model,
    messages: Vec<MessageContent>,
    temperature: Option<f32>,
}

#[derive(Debug, Deserialize, Clone)]
struct ChatChoice {
    message: MessageContent,
}

#[derive(Debug, Deserialize, Clone)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Default)]
struct Conversation {
    history: Vec<MessageContent>,
}

impl Conversation {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            history: vec![MessageContent {
                role: Role::System,
                content: prompt.into(),
            }],
        }
    }

    pub async fn message(&mut self, content: impl Into<String>) -> Result<String> {
        self.history.push(MessageContent {
            role: Role::User,
            content: content.into(),
        });

        let response = self.fetch_response().await?;
        Ok(response.content)
    }

    async fn fetch_response(&mut self) -> Result<MessageContent> {
        let token =
            env::var(OPENAI_API_TOKEN).expect(&format!("no {OPENAI_API_TOKEN} found in env"));

        let body = ChatCompletion {
            model: Model::Gpt35Turbo,
            messages: self.history.clone(),
            temperature: None,
        };
        let ser_body = serde_json::to_string(&body)?;

        let client = reqwest::Client::new();
        let res = client
            .post(COMPLETIONS_URL)
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(ser_body)
            .send()
            .await?;

        let response: ChatCompletionResponse = res.json().await?;
        let message = response
            .choices
            .first()
            .expect("no message returned")
            .message
            .clone();

        self.history.push(message.clone());

        Ok(message)
    }
}

// const CONVERSATION_PROMPT: &str = "I am learning to speak Polish. You are a Polish teacher. Let's have a conversation at A2 level in Polish.";
// const TEACH_PROMPT: &str = "I am learning to speak Polish. You are a Polish teacher. Please correct any grammar or mistakes I make in the following sentences, in English. If there are no mistakes, just say 'All good'. Please only speak in English.";
