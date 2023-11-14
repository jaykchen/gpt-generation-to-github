use async_openai_wasi::{
    types::{ChatCompletionToolArgs, ChatCompletionToolType,
        ChatCompletionFunctionsArgs, ChatCompletionRequestFunctionMessageArgs,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, FinishReason, Role,
    },
    Client,
};
use chrono::prelude::*;
use dotenv::dotenv;
use serde_json::json;
use slack_flows::{listen_to_channel, send_message_to_channel};
use std::collections::HashMap;
use std::env;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
async fn run() {
    dotenv().ok();
    let slack_workspace = env::var("slack_workspace").unwrap_or("secondstate".to_string());
    let slack_channel = env::var("slack_channel").unwrap_or("test-flow".to_string());

    listen_to_channel(&slack_workspace, &slack_channel, |sm| {
        handler(&slack_workspace, &slack_channel, sm.text)
    })
    .await;
}

#[no_mangle]
async fn handler(workspace: &str, channel: &str, msg: String) {
    let trigger_word = env::var("trigger_word").unwrap_or("tool_calls".to_string());

    match msg.starts_with(&trigger_word) {
        false => {}

        true => {
            let user_input = msg.replace(&trigger_word, "").to_string();

            let _ = run_gpt(workspace, channel, user_input).await;
        }
    }
}

pub async fn run_gpt(
    workspace: &str,
    channel: &str,
    user_input: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut messages = vec![
        ChatCompletionRequestSystemMessageArgs::default()
            .content("Perform function requests for the user")
            .build()?
            .into(),
        ChatCompletionRequestUserMessageArgs::default()
            .content(user_input)
            // .content("Hello, I am a user, I would like to know the time of day now")
            .build()?
            .into(),
    ];

    let tools = vec![
        ChatCompletionToolArgs::default()
            .r#type(ChatCompletionToolType::Function)
            .function(
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
            )
            .build()?,
        ChatCompletionToolArgs::default()
            .r#type(ChatCompletionToolType::Function)
            .function(
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
            )
            .build()?,
        ChatCompletionToolArgs::default()
            .r#type(ChatCompletionToolType::Function)
            .function(
                ChatCompletionFunctionsArgs::default()
                    .name("getTimeOfDay")
                    .description("Get the time of day.")
                    .parameters(json!({
                        "type": "object",
                        "properties": {},
                        "required": [],
                    }))
                    .build()?,
            )
            .build()?,
    ];

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo-0613")
        .messages(messages.clone())
        .tools(tools)
        .build()?;

    let chat = client.chat().create(request).await?;

    let wants_to_use_function = chat
        .choices
        .get(0)
        .map(|choice| choice.finish_reason == Some(FinishReason::FunctionCall))
        .unwrap_or(false);

    if wants_to_use_function {
        let tool_calls = chat.choices[0].message.tool_calls.as_ref().unwrap();

        for tool_call in tool_calls {
            let function = &tool_call.function;
            let content = match function.name.as_str() {
                "helloWorld" => {
                    let argument_obj =
                        serde_json::from_str::<HashMap<String, String>>(&function.arguments)?;
                    hello_world(argument_obj["appendString"].clone())
                }
                "scraper" => {
                    let argument_obj =
                        serde_json::from_str::<HashMap<String, String>>(&function.arguments)?;
                    scraper(argument_obj["keyword"].clone()).await?
                }
                "getTimeOfDay" => get_time_of_day(),
                _ => "".to_string(),
            };
            messages.push(
                ChatCompletionRequestFunctionMessageArgs::default()
                    .role(Role::Function)
                    .name(function.name.clone())
                    .content(content)
                    .build()?
                    .into(),
            );
        }
    }

    let response_after_func_run = client
        .chat()
        .create(
            CreateChatCompletionRequestArgs::default()
                .model("gpt-3.5-turbo-0613")
                .messages(messages)
                .build()?,
        )
        .await?;

    let res = response_after_func_run
        .choices
        .get(0)
        .unwrap()
        .message
        .clone()
        .content
        .unwrap_or("no result".to_string());
    send_message_to_channel(workspace, channel, res).await;

    Ok(())
}

fn hello_world(append_string: String) -> String {
    format!("Hello, world! {}", append_string)
}

async fn scraper(keyword: String) -> Result<String, Box<dyn std::error::Error>> {
    Ok(format!("Scraped books with keyword: {}", keyword))
}

fn get_time_of_day() -> String {
    let now = Local::now();
    now.to_rfc3339()
}
