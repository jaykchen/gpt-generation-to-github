use dotenv::dotenv;
// use openai_flows::{
//     chat::{ChatOptions,ChatModel, ResponseFormat, ResponseFormatType},
//      OpenAIFlows,
// };

// // use github_flows::{get_octo, octocrab::Octocrab, GithubLogin};
// use slack_flows::{listen_to_channel, send_message_to_channel, SlackMessage};
use std::{env, fmt::format};
use async_openai::{Chat};

#[no_mangle]
#[tokio::main]
pub async fn run() {
    dotenv().ok();
    let workspace: String = match env::var("slack_workspace") {
        Err(_) => "secondstate".to_string(),
        Ok(name) => name,
    };

    let channel: String = match env::var("slack_channel") {
        Err(_) => "collaborative-chat".to_string(),
        Ok(name) => name,
    };

    listen_to_channel(&workspace, &channel, |sm| handler(sm, &workspace, &channel)).await;
}

async fn handler(sm: SlackMessage, workspace: &str, channel: &str) {
    let chat_id = workspace.to_string() + channel;
    let co = ChatOptions {
        model: ChatModel::GPT4Turbo,
        restart: false,
        system_prompt: None,
        response_format: Some(ResponseFormat {
            r#type: ResponseFormatType::JsonObject,
        }),
        ..Default::default()
    };
    let openai = OpenAIFlows::new();
    if let Ok(c) = openai.chat_completion(&chat_id, &sm.text, &co).await {
        send_message_to_channel(&workspace, &channel, c.choice).await;
    }
}

/* {
   "model": "gpt-3.5-turbo",
   "messages": [
     {
       "role": "user",
       "content": "Generate content for a README file for my new project."
     }
   ],
   "functions": [
     {
       "name": "upload_readme",
       "description": "Uploads a README file to a given GitHub repository",
       "parameters": {
         "type": "object",
         "properties": {
           "owner": {
             "type": "string",
             "description": "The owner of the repository"
           },
           "repo": {
             "type": "string",
             "description": "The name of the repository"
           },
           "file_name": {
             "type": "string",
             "description": "The name of the file"
           },
           "file_content": {
             "type": "string",
             "description": "The content of the file"
           }
         },
         "required": ["owner", "repo", "file_name", "file_content"]
       }
     }
   ],
   "function_call": "auto"
 }
*/
pub async fn upload_readme(owner: &str, repo: &str, file_name: &str, file_content: &str) {
    // let owner = "jaykchen";
    // let repo = "a-test";
    // let file_name = "README.md";
    // let octocrab = get_octo(&GithubLogin::Default);

    // let path = format!("{repo}/{file_name}");
    // let message = "gpt generated stuff";
    // let content = "blahblahblah".as_bytes().to_vec();

    // octocrab
    //     .repos(owner, repo)
    //     .create_file(path, message, content)
    //     .branch("master")
    //     .commiter(GitUser {
    //         name: "jaykchen".to_string(),
    //         email: "jaykchen@gmail.com".to_string(),
    //     })
    //     .author(GitUser {
    //         name: "jaykchen".to_string(),
    //         email: "jaykchen@gmail.com".to_string(),
    //     })
    //     .send()
    //     .await?;
}
