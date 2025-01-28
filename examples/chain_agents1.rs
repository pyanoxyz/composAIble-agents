use std::{ collections::HashMap, error::Error as StdError };
use axum::Json;
use pyano::{
    llm::options::LLMHTTPCallOptions,
    agent::agent_builder::AgentBuilder,
    chain::sequential_chain::Chain,
    ModelManager,
};
use log::{ info, error };
use std::sync::{ Arc, Mutex };

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    // Initialize logging
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    info!("Initializing ModelManager");
    let model_manager = Arc::new(ModelManager::new());
    model_manager.show_registry(); // get_model_registry
    info!("Gettig SmolTalk model");
    let content_llm = model_manager
        .clone()
        .get_llm("smolTalk", None).await
        .map_err(|e| {
            error!("Failed to Get SmolTalk model: {}", e);
            e
        })?;

    info!("Loading SmolTalk model");
    content_llm.clone().load().await;
    info!("SmolTalk Model loaded");

    let prompt_template =
        "
        <|begin_of_text|><|start_header_id|>system<|end_header_id|>
            Cutting Knowledge Date: December 2023
            Today Date: 26 Jul 2024
        {system_prompt}<|eot_id|><|start_header_id|>user<|end_header_id|>
        {user_prompt}<|eot_id|><|start_header_id|>assistant<|end_header_id|>
    ";

    let options = LLMHTTPCallOptions::new()
        .with_server_url("http://localhost:5010".to_string()) // Add complete server URL
        .with_port(5010)
        .with_temperature(0.8)
        .with_prompt_template(prompt_template.to_string())
        .build();

    // Update this to return LLM Only Remove ModelRequest
    info!("Gettig Granite model");
    let llama_llm = model_manager
        .clone()
        .get_llm("granite", Some(options)).await
        .map_err(|e| {
            error!("Failed to Get Granite model: {}", e);
            e
        })?;

    info!("Loading Granite model");
    llama_llm.clone().load().await;
    // Create agents
    let agent_1 = Arc::new(
        Mutex::new(
            AgentBuilder::new()
                .with_name(String::from("Content Generator Agent"))
                .with_system_prompt("You are an excellent content generator.".to_string())
                .with_user_prompt(
                    "Generate content on the topic - Future of AI agentix framework".to_string()
                )
                .with_stream(true)
                .with_llm(content_llm)
                .build()
        )
    );
    // Get LLM for LLaMA (Qwen will be unloaded if memory is low)
    let agent_2 = Arc::new(
        Mutex::new(
            AgentBuilder::new()
                .with_name(String::from("Analyzer Agent"))
                .with_system_prompt("You are a great analyzer of generated content.".to_string())
                .with_user_prompt("Analyze the generated content.".to_string())
                .with_stream(true)
                .with_llm(llama_llm)
                .build()
        )
    );
    // Create a chain and add agents
    let mut chain = Chain::new().add_agent(agent_1).add_agent(agent_2);
    // Run the chain
    if let Err(e) = chain.run().await {
        eprintln!("Error executing chain: {}", e);
    }
    model_manager.show_model_details().await;

    // Access the memory logs
    let logs = chain.memory_logs();
    for log in logs {
        println!(
            "Agent: {}, Input: {}, Output: {}, Timestamp: {:?}",
            log.agent_name,
            log.input,
            log.output,
            log.timestamp
        );
    }
    Ok(())
}
