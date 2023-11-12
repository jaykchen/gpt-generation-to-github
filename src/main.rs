// use async_openai::types::assistants::thread::{
//     , CreateThreadRequestArgs,
// };

use async_openai::{
    types::{ChatCompletionFunctionsArgs,ChatCompletionRequestFunctionMessageArgs,
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, FinishReason, Role,
    },
    Client,
};
use chrono::prelude::*;
use serde_json::json;
use std::collections::HashMap;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut messages = vec![
        ChatCompletionRequestSystemMessageArgs::default()
            .content("Perform function requests for the user")
            .build()?
            .into(),
        ChatCompletionRequestUserMessageArgs::default()
            .content("Hello, I am a user, I would like to know the time of day now")
            .build()?
            .into(),
    ];
    let functions = vec![
        ChatCompletionFunctionsArgs::default()
            .name("helloWorld")
            .description("Prints hello world with the string passed to it")
            .parameters(json!({
                "type": "object",
                "properties": {
                    "appendString": {
                        "type": "string",
                        "description": "The string to append to the hello world message",
                    },
                },
                "required": ["appendString"],
            }))
            .build()?,
        ChatCompletionFunctionsArgs::default()
            .name("scraper")
            .description(
                "Scraps the book website goodreads for books with the keyword passed to it",
            )
            .parameters(json!({
                "type": "object",
                "properties": {
                    "keyword": {
                        "type": "string",
                        "description": "The keyword to search for",
                    },
                },
                "required": ["keyword"],
            }))
            .build()?,
        ChatCompletionFunctionsArgs::default()
            .name("getTimeOfDay")
            .description("Get the time of day.")
            .parameters(json!({
                "type": "object",
                "properties": {},
                "required": [],
            }))
            .build()?,
    ];

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo-0613")
        .messages(messages.clone())
        .functions(functions)       
        .function_call("auto")
        .build()?;

    let chat = client.chat().create(request).await?;

    let wants_to_use_function = chat
        .choices
        .get(0)
        .map(|choice| choice.finish_reason == Some(FinishReason::FunctionCall))
        .unwrap_or(false);
    if wants_to_use_function {
        let function_call = chat.choices[0].message.function_call.as_ref().unwrap();
        let content = match function_call.name.as_str() {
            "helloWorld" => {
                let argument_obj = serde_json::from_str::<HashMap<String, String>>(
                    &function_call.arguments,
                )?;
                hello_world(argument_obj["appendString"].clone())
            }
            "scraper" => {
                let argument_obj = serde_json::from_str::<HashMap<String, String>>(
                    &function_call.arguments,
                )?;
                scraper(argument_obj["keyword"].clone()).await?
            }
            "getTimeOfDay" => get_time_of_day(),
            _ => "".to_string(),
        };
        messages.push(
            ChatCompletionRequestFunctionMessageArgs::default()
                .role(Role::Function)
                .name(function_call.name.clone())
                .content(content)
                .build()?
                .into(),
        );
    }
    let step4response = client
        .chat().create(
            CreateChatCompletionRequestArgs::default()
                .model("gpt-3.5-turbo-0613")
                .messages(messages)
                .build()?,
        )
        .await?;
    println!("{:?}", step4response.choices.get(0));
    Ok(())
}

fn hello_world(append_string: String) -> String {
    format!("Hello, world! {}", append_string)
}

async fn scraper(keyword: String) -> Result<String, Box<dyn std::error::Error>> {
    // Implement the scraper function here.
    Ok(format!("Scraped books with keyword: {}", keyword))
}

fn get_time_of_day() -> String {
    let now = Local::now();
    now.to_rfc3339()
}
