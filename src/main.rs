use async_openai::{
    types::{
        ChatCompletionFunctionsArgs, ChatCompletionNamedToolChoice,
        ChatCompletionRequestFunctionMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
        ChatCompletionToolArgs, ChatCompletionToolChoiceOption, ChatCompletionToolType,
        CreateChatCompletionRequestArgs, FinishReason, FunctionName, Role,
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
            // .content("Hello, I am a user, use your predefined functions to scrape the internet for information about the Old Man and the Sea")
            // .content("Hello, I am a user, I would like to greet John")
            .content("Hello, I am a user, use your predefined functions to get the time of day now")
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
                    .description(
                        "A function to get the time of day that runs locally on client's PC.",
                    )
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
        // .model("gpt-4-1106-preview")
        .model("gpt-3.5-turbo-1106")
        .messages(messages.clone())
        .tools(tools)
        // .tool_choice(ChatCompletionToolChoiceOption::Named(
        //     ChatCompletionNamedToolChoice {
        //         r#type: ChatCompletionToolType::Function,
        //         function: FunctionName {
        //             name: "getTimeOfDay".to_string(),
        //         },
        //     },
        // ))
        // .tool_choice(ChatCompletionToolChoiceOption::None)
        // .tool_choice(ChatCompletionToolChoiceOption::Auto)
        .build()?;

    let chat = client.chat().create(request).await?;

    let wants_to_use_function = chat
        .choices
        .get(0)
        .map(|choice| choice.finish_reason == Some(FinishReason::FunctionCall))
        .unwrap_or(false);

    let mut messages_log = String::new();

    if wants_to_use_function {

        let tool_calls = chat.choices[0].message.tool_calls.as_ref().unwrap();

        for tool_call in tool_calls {
            let function = &tool_call.function;
            let content = match function.name.as_str() {
                "helloWorld" => {
                    let argument_obj =
                        serde_json::from_str::<HashMap<String, String>>(&function.arguments)?;

                    let appendString = &argument_obj["appendString"];

                    hello_world(argument_obj["appendString"].clone())
                }
                "scraper" => {
                    let argument_obj =
                        serde_json::from_str::<HashMap<String, String>>(&function.arguments)?;

                    let keyword = &argument_obj["keyword"];

                    scraper(argument_obj["keyword"].to_string()).await.unwrap()
                }
                "getTimeOfDay" => get_time_of_day(),
                _ => "".to_string(),
            };
            messages.push(
                ChatCompletionRequestToolMessageArgs::default()
                    .role(Role::Tool)
                    .content(content)
                    .build()?
                    .into(),
                // ChatCompletionRequestFunctionMessageArgs::default()
                //     .role(Role::Function)
                //     .name(function.name.clone())
                //     .content(content)
                //     .build()?
                //     .into(),
            );
        }
    }
    let response_after_func_run = client
        .chat()
        .create(
            CreateChatCompletionRequestArgs::default()
                // .model("gpt-4-1106-preview")
                .model("gpt-3.5-turbo-1106")
                .messages(messages.clone())
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
    println!("{:?}", res);

    messages.push(
        ChatCompletionRequestUserMessageArgs::default()
            .content(res)
            .build()?
            .into(),
    );

    let final_res = client
        .chat()
        .create(
            CreateChatCompletionRequestArgs::default()
                // .model("gpt-4-1106-preview")
                .model("gpt-3.5-turbo-1106")
                .messages(messages)
                .build()?,
        )
        .await?;

    let res = final_res
        .choices
        .get(0)
        .unwrap()
        .message
        .clone()
        .content
        .unwrap_or("no result".to_string());
    println!("{:?}", res);

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
