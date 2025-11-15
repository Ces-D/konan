use anyhow::{Result, bail};
use async_openai::{
    Client,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs,
    },
};

const ASCII_ART_SYSTEM_PROMPT: &str = "You are an ASCII art generator.
The user wil provide an object, scene, or character, produce ASCII art that clearly resembles the request while following these strict rules:
1. Use only ASCII characters (letters, numbers, symbols found on a standard keyboard).
2.Maximum width: 47 characters per line.
3.The art should be recognizable, centered, and cleanly formatted inside a code block.
4.Do not include explanations, descriptions, or extra text—only the ASCII art.
5.Avoid trailing spaces.";

pub async fn generate_ascii_art(art_query: &str, model: &str) -> Result<String> {
    let client = Client::new();
    let messages = vec![
        ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
            content: ChatCompletionRequestSystemMessageContent::Text(
                ASCII_ART_SYSTEM_PROMPT.to_string(),
            ),
            ..Default::default()
        }),
        ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Text(art_query.to_string()),
            ..Default::default()
        }),
    ];
    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(messages)
        .build()?;
    let response = client.chat().create(request).await?;
    match response.choices.first() {
        Some(choice) => Ok(choice
            .clone()
            .message
            .content
            .expect("Expected a message response from the model")),
        None => bail!("No response from OpenAI"),
    }
}
