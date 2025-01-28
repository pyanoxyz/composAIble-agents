use std::pin::Pin;

use log::info;
use bytes::Bytes;

// use serde_json::json;
use serde::Deserialize;

use futures::{ Stream, StreamExt }; // Ensure StreamExt is imported
use serde_json::Value;

type StreamResult = Result<Bytes, reqwest::Error>;
type BoxedStream = Pin<Box<dyn Stream<Item = StreamResult> + Send>>;
use colored::Colorize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct LLMGenerattionTimings {
    predicted_ms: f64,
    predicted_n: f64,
    predicted_per_second: f64,
    predicted_per_token_ms: f64,
    prompt_ms: f64,
    prompt_n: f64,
    prompt_per_second: f64,
    prompt_per_token_ms: f64,
}

pub fn llamacpp_process_stream<'a>(stream: BoxedStream) -> BoxedStream {
    Box::pin(
        futures::stream::unfold((stream, String::new()), |(mut stream, acc)| async move {
            if let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if let Ok(chunk_str) = std::str::from_utf8(&chunk) {
                            let content_to_stream = process_chunk(chunk_str).await;

                            if !content_to_stream.is_empty() {
                                return Some((Ok(Bytes::from(content_to_stream)), (stream, acc)));
                            }
                        } else {
                            eprintln!("Failed to parse chunk as UTF-8");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving chunk: {}", e);
                        return Some((Err(e), (stream, acc)));
                    }
                }
            } else {
                return None;
            }

            Some((Ok(Bytes::new()), (stream, acc)))
        })
    )
}

pub fn qwen_process_stream(stream: BoxedStream) -> BoxedStream {
    // For now, using the same implementation as llamacpp
    llamacpp_process_stream(stream)
}

async fn process_chunk(chunk_str: &str) -> String {
    let mut content_to_stream = String::new();

    for line in chunk_str.lines() {
        if line.starts_with("data: ") {
            if let Ok(json_data) = serde_json::from_str::<Value>(&line[6..]) {
                if let Some(content) = json_data.get("content").and_then(|c| c.as_str()) {
                    content_to_stream.push_str(content); // Stream content
                }
                if let Some(timings) = json_data.get("timings") {
                    if
                        let Ok(timing_struct) = serde_json::from_value::<LLMGenerattionTimings>(
                            timings.clone()
                        )
                    {
                        let tokens_per_second = calculate_tokens_per_second(
                            timing_struct.predicted_n,
                            timing_struct.predicted_ms
                        );
                        println!("");
                        info!(
                            "Tokens generated per second: {:.2}",
                            tokens_per_second.to_string().yellow()
                        );
                    }
                }
            }
        }
    }
    content_to_stream
}

fn calculate_tokens_per_second(predicted_n: f64, predicted_ms: f64) -> f64 {
    let predicted_seconds = predicted_ms / 1000.0;
    predicted_n / predicted_seconds
}
