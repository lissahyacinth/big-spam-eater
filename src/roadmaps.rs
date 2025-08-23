use crate::consts::MODEL_USED;
use anyhow::bail;
use lazy_static::lazy_static;
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use serde::Deserialize;
use tracing::info;
lazy_static! {
    static ref ROADMAP_CONFIG: RoadmapConfig = RoadmapConfig::default();
}

static DETECT_ROADMAP_PROMPT: &str = include_str!("../prompts/detect_roadmap.txt");

static CREATE_ROADMAP_PROMPT: &str = include_str!("../prompts/create_roadmap_for_user.txt");

#[derive(Deserialize)]
struct RoadmapConfig {
    context_length: usize,
    message_limit_chars: usize,
}

impl Default for RoadmapConfig {
    fn default() -> Self {
        RoadmapConfig {
            context_length: 3,
            message_limit_chars: 2048,
        }
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct RequestingRoadmap {
    pub reason: String,
    #[allow(dead_code)]
    pub is_roadmap: bool,
}

#[derive(Deserialize, Debug)]
pub(crate) struct RoadmapProvided {
    pub roadmap: String,
}

fn system_message_detection() -> ChatCompletionMessage {
    ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(DETECT_ROADMAP_PROMPT.to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        function_call: None,
    }
}

fn system_message_creation() -> ChatCompletionMessage {
    ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(CREATE_ROADMAP_PROMPT.to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        function_call: None,
    }
}

fn user_message(message: String) -> ChatCompletionMessage {
    ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(message),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        function_call: None,
    }
}

fn build_message(
    message: String,
    context: Vec<String>,
    system_message: ChatCompletionMessage,
) -> Vec<ChatCompletionMessage> {
    let mut messages: Vec<ChatCompletionMessage> = vec![system_message];
    let mut message_length: usize = message.len();
    let mut message_buffer: String = message;
    for contextual_message in context.into_iter().take(ROADMAP_CONFIG.context_length) {
        if message_length + contextual_message.len() > ROADMAP_CONFIG.message_limit_chars {
            break;
        }
        message_length += contextual_message.len();
        message_buffer.insert_str(0, contextual_message.as_str());
    }
    messages.push(user_message(message_buffer));
    messages
}

pub(crate) async fn is_message_roadmap_request(
    message: String,
    context: Vec<String>,
) -> anyhow::Result<RequestingRoadmap> {
    let chat_completion = ChatCompletion::builder(
        MODEL_USED,
        build_message(message.clone(), context, system_message_detection()),
    )
    .create()
    .await?;
    let returned_message = chat_completion.choices.first().unwrap().message.clone();
    if let Some(content) = returned_message.content {
        let roadmap_request: RequestingRoadmap = serde_json::from_str(content.as_str())?;
        if roadmap_request.is_roadmap {
            info!(
                "Generating roadmap for request {} due to {}",
                message.as_str(),
                roadmap_request.reason.as_str()
            );
        } else {
            info!(
                "Ignoring roadmap request {} due to {}",
                message.as_str(),
                roadmap_request.reason.as_str()
            );
        }
        Ok(roadmap_request)
    } else {
        bail!("No reply from ChatGPT")
    }
}

pub(crate) async fn create_roadmap(
    message: String,
    context: Vec<String>,
) -> anyhow::Result<RoadmapProvided> {
    let chat_completion = ChatCompletion::builder(
        MODEL_USED,
        build_message(message, context, system_message_creation()),
    )
    .create()
    .await?;
    let returned_message = chat_completion.choices.first().unwrap().message.clone();
    if let Some(content) = returned_message.content {
        info!("Generated Roadmap - {}", content.as_str());
        Ok(RoadmapProvided { roadmap: content })
    } else {
        bail!("No reply from ChatGPT")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_prompt() {
        dbg!(build_message(
            "I'd like a roadmap".to_string(),
            vec![],
            system_message_creation()
        ));
    }
}
