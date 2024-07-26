use anyhow::bail;
use config::Config;
use lazy_static::lazy_static;
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use serde::Deserialize;

lazy_static! {
    static ref SPAM_CONFIG: SpamConfig = {
        Config::builder()
            .add_source(config::Environment::default())
            .build()
            .expect("Could not build SpamConfig")
            .try_deserialize()
            .expect("Could not deserialize into SpamConfig")
    };
}

static SPAM_PROMPT: &str = include_str!("../prompts/spam_role.txt");

#[derive(Deserialize)]
struct SpamConfig {
    context_length: usize,
    message_limit_chars: usize,
}

impl Default for SpamConfig {
    fn default() -> Self {
        SpamConfig {
            context_length: 3,
            message_limit_chars: 2048,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct IsSpamResult {
    pub reason: String,
    pub is_spam: bool,
}

fn system_message() -> ChatCompletionMessage {
    ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(SPAM_PROMPT.to_string()),
        name: None,
        function_call: None,
    }
}

fn user_message(message: String) -> ChatCompletionMessage {
    ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(message),
        name: None,
        function_call: None,
    }
}

fn build_message(message: String, context: Vec<String>) -> Vec<ChatCompletionMessage> {
    let mut messages: Vec<ChatCompletionMessage> = vec![system_message()];
    let mut message_length: usize = message.len();
    let mut message_buffer: String = message;
    for contextual_message in context.into_iter().take(SPAM_CONFIG.context_length) {
        if message_length + contextual_message.len() > SPAM_CONFIG.message_limit_chars {
            break;
        }
        message_length += contextual_message.len();
        message_buffer.insert_str(0, contextual_message.as_str());
    }
    messages.push(user_message(message_buffer));
    messages
}

pub(crate) async fn classify_message_spam(
    message: String,
    context: Vec<String>,
) -> anyhow::Result<IsSpamResult> {
    let chat_completion = ChatCompletion::builder("gpt-4o-mini", build_message(message, context))
        .create()
        .await
        .unwrap();
    let returned_message = chat_completion.choices.first().unwrap().message.clone();
    if let Some(content) = returned_message.content {
        Ok(serde_json::from_str(content.as_str())?)
    } else {
        bail!("No reply from ChatGPT")
    }
}
