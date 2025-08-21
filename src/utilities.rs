use openai::chat::{ChatCompletionMessage, ChatCompletionMessageRole};

pub(crate) fn user_message(message: String) -> ChatCompletionMessage {
    ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(message),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        function_call: None,
    }
}

pub(crate) fn build_message(
    message: String,
    context: Vec<String>,
    system_message: ChatCompletionMessage,
    context_length: usize,
    message_limit_chars: usize,
) -> Vec<ChatCompletionMessage> {
    let mut messages: Vec<ChatCompletionMessage> = vec![system_message];
    let mut message_length: usize = message.len();
    for contextual_message in context.into_iter().take(context_length) {
        if message_length + contextual_message.len() > message_limit_chars {
            break;
        }
        message_length += contextual_message.len();
        messages.push(user_message(contextual_message));
    }
    messages.push(user_message(message));
    messages
}
