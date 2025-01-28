use std::collections::HashMap;
use log::{ info, debug };
use crate::model::utils::get_env_var;

use super::{
    ModelConfig,
    ModelDefaults,
    ModelMemoryConfig,
    ModelSpecificConfig,
    PromptTemplate,
    ServerConfig,
};

pub struct ModelRegistry {
    configs: HashMap<String, ModelConfig>,
}

use std::fs;
use serde_json::Value;

impl ModelRegistry {
    pub fn new() -> Self {
        debug!("Initializing ModelRegistry");
        let configs = Self::load_configs_from_json();
        Self { configs }
    }

    fn load_configs_from_json() -> HashMap<String, ModelConfig> {
        let mut configs = HashMap::new();
        let config_dir = get_env_var("MODEL_CONFIG_DIR").unwrap_or(
            "pyano_home/configs".to_string()
        );

        debug!("Loading model configurations from {}", config_dir);

        for entry in fs::read_dir(&config_dir).expect("Failed to read config directory") {
            let path = entry.expect("Failed to read entry").path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                debug!("Processing config file: {:?}", path);

                let config_str = fs::read_to_string(&path).expect("Failed to read config file");

                let json: Value = serde_json::from_str(&config_str).expect("Failed to parse JSON");

                debug!("Parsed JSON structure: {:#?}", json);

                // Extract base configurations
                let model_config: ModelSpecificConfig = json
                    .get("model_config")
                    .and_then(|v| serde_json::from_value::<ModelSpecificConfig>(v.clone()).ok())
                    .expect("Model config is required");
                let memory_config = json
                    .get("memory_config")
                    .and_then(|v| serde_json::from_value::<ModelMemoryConfig>(v.clone()).ok())
                    .expect("Memory config is required");
                let prompt_template = json
                    .get("prompt_template")
                    .and_then(|v| serde_json::from_value::<PromptTemplate>(v.clone()).ok())
                    .expect("Prompt template is required");

                let defaults = json
                    .get("defaults")
                    .and_then(|v| serde_json::from_value::<ModelDefaults>(v.clone()).ok())
                    .expect("Defaults are required");

                let server_config = json
                    .get("server_config")
                    .and_then(|v| serde_json::from_value::<ServerConfig>(v.clone()).ok())
                    .expect("Server config is required");

                // Get model details directly

                let name = model_config.name.clone();

                debug!("Processing model: {}", name);

                let config = ModelConfig {
                    model_config: model_config.clone(),
                    memory_config: memory_config.clone(),
                    prompt_template: prompt_template.clone(),
                    defaults: defaults.clone(),
                    server_config: server_config.clone(),
                };

                debug!("Adding model to registry: {}", name);
                configs.insert(name.to_string(), config);
                debug!("Loaded configuration for model: {}", name);
            }
        }

        debug!("Loaded {} model configurations", configs.len());
        configs
    }

    pub fn get_config(&self, model_name: &str) -> Option<&ModelConfig> {
        let config = self.configs.get(model_name);
        if config.is_none() {
            info!("Configuration not found for model: {}", model_name);
        }
        config
    }

    pub fn get_all_configs(&self) -> Vec<(&String, &ModelConfig)> {
        self.configs.iter().collect()
    }
}
