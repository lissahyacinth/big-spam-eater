use openai::chat::{ChatCompletionMessage, ChatCompletionMessageRole};

pub(crate) fn user_message(message: String) -> ChatCompletionMessage {
    ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(message),
        name: None,
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
    let mut message_buffer: String = message;
    for contextual_message in context.into_iter().take(context_length) {
        if message_length + contextual_message.len() > message_limit_chars {
            break;
        }
        message_length += contextual_message.len();
        message_buffer.insert_str(0, contextual_message.as_str());
    }
    messages.push(user_message(message_buffer));
    messages
}
