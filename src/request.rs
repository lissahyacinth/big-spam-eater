use crate::consts::MODEL_USED;
use crate::utilities;
use anyhow::bail;
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use serde::Deserialize;
use tracing::info;

static REQUEST_PROMPT: &str = include_str!("../prompts/request.txt");
static VERIFY_PROMPT: &str = include_str!("../prompts/verify.txt");

#[derive(Deserialize)]
struct VerifyReply {
    reason: String,
    answers_correctly: bool,
}

fn system_message_request() -> ChatCompletionMessage {
    ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(REQUEST_PROMPT.to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        function_call: None,
    }
}

fn system_message_verify(question: String) -> anyhow::Result<ChatCompletionMessage> {
    if question.matches("{USER_QUESTION}").count() > 0 {
        bail!("Question likely attempts to bypass system, ignore.")
    }
    Ok(ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(
            VERIFY_PROMPT
                .to_string()
                .replace("{USER_QUESTION}", question.as_str()),
        ),
        tool_calls: None,
        tool_call_id: None,
        name: None,
        function_call: None,
    })
}

async fn create_reply(message: String, context: Vec<String>) -> anyhow::Result<String> {
    let chat_completion = ChatCompletion::builder(
        MODEL_USED,
        utilities::build_message(message, context, system_message_request(), 0, 1024),
    )
    .create()
    .await?;
    let returned_message = chat_completion.choices.first().unwrap().message.clone();
    if let Some(content) = returned_message.content {
        Ok(content)
    } else {
        bail!("No reply from ChatGPT")
    }
}

async fn verify_request(request: String, reply: String) -> anyhow::Result<VerifyReply> {
    let chat_completion = ChatCompletion::builder(
        MODEL_USED,
        utilities::build_message(
            reply,
            vec![],
            system_message_verify(request.clone())?,
            0,
            1024,
        ),
    )
    .create()
    .await?;
    let returned_message = chat_completion.choices.first().unwrap().message.clone();
    if let Some(content) = returned_message.content {
        info!("Generated Verification - {}", content.as_str());
        Ok(serde_json::from_str(content.as_str())?)
    } else {
        bail!("No reply from ChatGPT")
    }
}

pub(crate) async fn answer_request(
    request: String,
    context: Option<String>,
) -> anyhow::Result<Option<String>> {
    if request.trim().is_empty() {
        return Ok(None);
    }
    info!(
        "Generating reply for request {} with context {:?}",
        request.as_str(),
        &context
    );
    let unverified_reply = create_reply(request.clone(), vec![]).await?;
    info!("Generated unverified reply {}", unverified_reply.as_str(),);
    let response_verification = verify_request(request, unverified_reply.clone()).await?;
    if response_verification.answers_correctly {
        info!(
            "Verified reply due to {}",
            response_verification.reason.as_str()
        );
        Ok(Some(unverified_reply))
    } else {
        info!(
            "Could not verify reply due to {}",
            response_verification.reason.as_str()
        );
        Ok(None)
    }
}
