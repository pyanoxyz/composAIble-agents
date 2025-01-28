use pyano::model::{ ModelManager, ModelManagerServer };
use rust_bert::models;
use std::sync::Arc;
use env_logger;

use log::{ info, error };
use pyano::model::config_loader::ModelRegistry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the manager
    env_logger::init();
    let manager = Arc::new(ModelManager::new());

    // List all models from registry
    // let models = manager.list_models().await?;
    let registery: ModelRegistry = ModelRegistry::new();
    //TODO : Add models in listmodels from configs
    println!("\nAvailable Models in Registry:");
    println!("---------------------------");

    manager.show_registry();
    // Print configs from ModelRegistry
    println!("\nFrom ModelRegistry:");
    // Try getting configs for known models
    for model_name in ["qwen-7b"] {
        if let Some(config) = registery.get_config(model_name) {
            println!(
                "Name: {}\nType: {:?}\nKind: {}\nPath: {:?}\nMemory: {:?} GB (min) / {:?} GB (recommended)\nPort: {:?}\n",
                config.model_config.name,
                config.model_config.model_type,
                config.model_config.model_kind,
                config.model_config.model_path,
                config.memory_config.min_ram_gb,
                config.memory_config.recommended_ram_gb,
                config.server_config.port
            );
        }
    }

    Ok(())
}
