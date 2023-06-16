use colored::*;
use indicatif::ProgressBar;
use serde_json;
use std::env;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termimad::{Area, MadView};

#[derive(serde::Deserialize, Debug)]
struct GptResponse {
    choices: Vec<Choice>,
}

#[derive(serde::Deserialize, Debug)]
struct Choice {
    message: Message,
    // finish_reason: String,
    // index: usize,
}

#[derive(serde::Deserialize, Debug)]
struct Message {
    // role: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the API parameters and prompt from the user
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    // let prompt = read_input("Enter your prompt: ")?;
    let args: Vec<String> = env::args().collect();
    let prompt = args[1..].join(" ");

    let model = "gpt-4";

    // Create an HTTP client
    let client = reqwest::Client::new();

    let messages = vec![
        // serde_json::json!({
        //     "role": "system",
        //     "content": "You are a helpful assistant."
        // }),
        serde_json::json!({
            "role": "user",
            "content": prompt
        }),
    ];

    let json_payload = serde_json::json!({
        "model": model,
        "messages": messages,
        "max_tokens": 500,
        "temperature": 0.5,
        "top_p": 1.0,
        "frequency_penalty": 0.5,
        "presence_penalty": 0.5
    });

    // Create a progress bar spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Thinking...".yellow().to_string());

    let (tx, rx) = mpsc::channel();

    // Spawn a separate thread to simulate the ticking behavior
    let pb_thread = thread::spawn(move || loop {
        spinner.tick();
        thread::sleep(Duration::from_millis(100));
        if rx.try_recv().is_ok() {
            break;
        }
    });

    // Send the API request
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        // .bearer_auth(&api_key)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json_payload)
        .send()
        .await?;

    // Read and print the response
    let response_json: GptResponse = response.json().await?;

    // Finish the spinner
    tx.send(()).unwrap();
    pb_thread.join().unwrap();

    // Create an Area to render the Markdown content
    let area = Area::full_screen();
    let view = MadView::from(
        response_json.choices[0].message.content.to_string(),
        area,
        Default::default(),
    );

    // Render the Markdown content to the terminal
    view.write().unwrap();

    Ok(())
}
