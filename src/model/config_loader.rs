use std::collections::HashMap;
use log::info;
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

// impl ModelRegistry {
//     pub fn new() -> Self {
//         info!("Initializing ModelRegistry");
//         let configs = Self::load_default_configs();
//         Self { configs }
//     }

//     fn load_default_configs() -> HashMap<String, ModelConfig> {
//         // Here we can either load from a JSON file or define in code
//         let mut configs = HashMap::new();

//         let model_path = std::env
//             ::var("QWEN_MODEL_PATH")
//             .unwrap_or_else(|_|
//                 "./models/Qwen2.5/Qwen2.5-Coder-7B-Instruct-Q6_K_L.gguf".to_string()
//             );
//         // Example Qwen config
//         configs.insert("qwen-7b".to_string(), ModelConfig {
//             name: "qwen-7b".to_string(),
//             model_type: ModelType::Text,
//             model_kind: "Qwen".to_string(),
//             model_path: PathBuf::from(model_path),
//             memory_config: ModelMemoryConfig {
//                 min_ram_gb: 1.0,
//                 recommended_ram_gb: 16.0,
//                 gpu_memory_gb: Some(8.0),
//             },
//             prompt_template: PromptTemplate {
//                 template: "<|im_start|>system\n{system}\n<|im_end|>\n<|im_start|>user\n{user}\n<|im_end|>".to_string(),
//                 required_keys: vec!["system".to_string(), "user".to_string()],
//             },
//             defaults: ModelDefaults {
//                 temperature: 0.7,
//                 top_p: 0.9,
//                 top_k: 40,
//                 max_tokens: 2048,
//                 repetition_penalty: 1.1,
//             },
//             server_config: ServerConfig {
//                 host: "localhost".to_string(),
//                 port: Some(8000),
//                 ctx_size: 4096,
//                 gpu_layers: -1,
//                 batch_size: 512,
//                 num_threads: Some(8),
//                 use_mmap: true,
//                 use_gpu: true,
//                 extra_args: HashMap::new(),
//             },
//         });

//         let llama_path = std::env
//             ::var("LLAMA_MODEL_PATH")
//             .unwrap_or_else(|_|
//                 "./models/Qwen2.5.1/Qwen2.5.1-Coder-7B-Instruct-Q4_0.gguf".to_string()
//             );

//         configs.insert("llama-7b".to_string(), ModelConfig {
//             name: "llama-7b".to_string(),
//             model_type: ModelType::Text,
//             model_kind: "LLaMa".to_string(),
//             model_path: PathBuf::from(llama_path),
//             memory_config: ModelMemoryConfig {
//                 min_ram_gb: 3.0,
//                 recommended_ram_gb: 16.0,
//                 gpu_memory_gb: Some(8.0),
//             },
//             prompt_template: PromptTemplate {
//                 template: "<|im_start|>system\n{system}\n<|im_end|>\n<|im_start|>user\n{user}\n<|im_end|>".to_string(),
//                 required_keys: vec!["system".to_string(), "user".to_string()],
//             },
//             defaults: ModelDefaults {
//                 temperature: 0.7,
//                 top_p: 0.9,
//                 top_k: 40,
//                 max_tokens: 2048,
//                 repetition_penalty: 1.1,
//             },
//             server_config: ServerConfig {
//                 host: "localhost".to_string(),
//                 port: Some(9001),
//                 ctx_size: 8096,
//                 gpu_layers: -1,
//                 batch_size: 512,
//                 num_threads: Some(8),
//                 use_mmap: true,
//                 use_gpu: true,
//                 extra_args: HashMap::new(),
//             },
//         });

//         let smol_talk_path = std::env
//             ::var("smolTalk_MODEL_PATH")
//             .unwrap_or_else(|_|
//                 "./models/Llama/Llama-SmolTalk-3.2-1B-Instruct-Q8_0.gguf".to_string()
//             );

//         configs.insert("smolTalk".to_string(), ModelConfig {
//             name: "smolTalk".to_string(),
//             model_type: ModelType::Text,
//             model_kind: "LLaMa".to_string(),
//             model_path: PathBuf::from(smol_talk_path),
//             memory_config: ModelMemoryConfig {
//                 min_ram_gb: 2.0,
//                 recommended_ram_gb: 16.0,
//                 gpu_memory_gb: Some(8.0),
//             },
//             prompt_template: PromptTemplate {
//                 template: "<|begin_of_text|><|start_header_id|>system<|end_header_id|>
// Cutting Knowledge Date: December 2023
// Today Date: 26 July 2024
// {system_prompt}<|eot_id|><|start_header_id|>user<|end_header_id|>
// {user_prompt}<|eot_id|><|start_header_id|>assistant<|end_header_id|>
// ".to_string(),
//                 required_keys: vec!["system_prompt".to_string(), "user_prompt".to_string()],
//             },
//             defaults: ModelDefaults {
//                 temperature: 0.7,
//                 top_p: 0.9,
//                 top_k: 40,
//                 max_tokens: 4096,
//                 repetition_penalty: 1.1,
//             },
//             server_config: ServerConfig {
//                 host: "localhost".to_string(),
//                 port: Some(5007),
//                 ctx_size: 16000,
//                 gpu_layers: -1,
//                 batch_size: 1024,
//                 num_threads: Some(8),
//                 use_mmap: true,
//                 use_gpu: true,
//                 extra_args: HashMap::new(),
//             },
//         });
//         let granite_path = std::env
//             ::var("Granite_MODEL_PATH")
//             .unwrap_or_else(|_| "./models/granite/granite-3.1-2b-instruct-Q6_K_L.gguf".to_string());

//         configs.insert("granite".to_string(), ModelConfig {
//             name: "granite".to_string(),
//             model_type: ModelType::Text,
//             model_kind: "LLaMa".to_string(),
//             model_path: PathBuf::from(granite_path),
//             memory_config: ModelMemoryConfig {
//                 min_ram_gb: 3.5,
//                 recommended_ram_gb: 16.0,
//                 gpu_memory_gb: Some(8.0),
//             },
//             prompt_template: PromptTemplate {
//                 template: "<<|start_of_role|>system<|end_of_role|>{system_prompt}<|end_of_text|>\n<|start_of_role|>user<|end_of_role|>{user_prompt}<|end_of_text|>\n<|start_of_role|>assistant<|end_of_role|>".to_string(),
//                 required_keys: vec!["system_prompt".to_string(), "user_prompt".to_string()],
//             },
//             defaults: ModelDefaults {
//                 temperature: 0.7,
//                 top_p: 0.9,
//                 top_k: 40,
//                 max_tokens: 2048,
//                 repetition_penalty: 1.1,
//             },
//             server_config: ServerConfig {
//                 host: "localhost".to_string(),
//                 port: Some(5008),
//                 ctx_size: 8096,
//                 gpu_layers: -1,
//                 batch_size: 512,
//                 num_threads: Some(8),
//                 use_mmap: true,
//                 use_gpu: true,
//                 extra_args: HashMap::new(),
//             },
//         });

//         info!("Loaded {} model configurations", configs.len());

//         configs
//     }

//     pub fn get_config(&self, model_name: &str) -> Option<&ModelConfig> {
//         let config = self.configs.get(model_name);
//         if config.is_none() {
//             info!("Configuration not found for model: {}", model_name);
//         }
//         config
//     }
// }

use std::fs;
use serde_json::Value;

// impl ModelRegistry {
//     pub fn new() -> Self {
//         info!("Initializing ModelRegistry");
//         let configs = Self::load_configs_from_json();
//         Self { configs }
//     }

//     fn load_configs_from_json() -> HashMap<String, ModelConfig> {
//         let mut configs = HashMap::new();
//         let config_dir = std::env
//             ::var("MODEL_CONFIG_DIR")
//             .unwrap_or_else(|_| "src/model/config".to_string());

//         info!("Loading model configurations from {}", config_dir);

//         let paths = fs
//             ::read_dir(&config_dir)
//             .unwrap_or_else(|_| panic!("Failed to read config directory: {}", config_dir));

//         for path in paths {
//             if let Ok(entry) = path {
//                 let path = entry.path();
//                 if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
//                     info!("Loading config from: {:?}", path);

//                     match fs::read_to_string(&path) {
//                         Ok(config_str) => {
//                             match serde_json::from_str::<Value>(&config_str) {
//                                 Ok(config_json) => {
//                                     if let Value::Object(models) = &config_json {
//                                         for (name, model_config) in models {
//                                             match serde_json::from_value(model_config.clone()) {
//                                                 Ok(config) => {
//                                                     configs.insert(name.clone(), config);
//                                                     info!("Loaded configuration for model: {}", name);
//                                                 }
//                                                 Err(e) => {
//                                                     info!(
//                                                         "Failed to parse config for {}: {}",
//                                                         name,
//                                                         e
//                                                     );
//                                                 }
//                                             }
//                                         }
//                                     }
//                                 }
//                                 Err(e) => info!("Failed to parse JSON from {:?}: {}", path, e),
//                             }
//                         }
//                         Err(e) => info!("Failed to read file {:?}: {}", path, e),
//                     }
//                 }
//             }
//         }

//         info!("Loaded {} model configurations", configs.len());
//         configs
//     }

//     pub fn get_config(&self, model_name: &str) -> Option<&ModelConfig> {
//         let config = self.configs.get(model_name);
//         if config.is_none() {
//             info!("Configuration not found for model: {}", model_name);
//         }
//         config
//     }
// }

// ToDo : Add a new method that implements uses the new_schema

impl ModelRegistry {
    pub fn new() -> Self {
        info!("Initializing ModelRegistry");
        let configs = Self::load_configs_from_json();
        Self { configs }
    }

    fn load_configs_from_json() -> HashMap<String, ModelConfig> {
        let mut configs = HashMap::new();
        let config_dir = get_env_var("MODEL_CONFIG_DIR").unwrap_or(
            "pyano_home/configs".to_string()
        );

        info!("Loading model configurations from {}", config_dir);

        for entry in fs::read_dir(&config_dir).expect("Failed to read config directory") {
            let path = entry.expect("Failed to read entry").path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                info!("Processing JSON file: {:?}", path);

                let config_str = fs::read_to_string(&path).expect("Failed to read config file");

                let json: Value = serde_json::from_str(&config_str).expect("Failed to parse JSON");

                info!("Parsed JSON structure: {:#?}", json);

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

                info!("Processing model: {}", name);

                let config = ModelConfig {
                    model_config: model_config.clone(),
                    memory_config: memory_config.clone(),
                    prompt_template: prompt_template.clone(),
                    defaults: defaults.clone(),
                    server_config: server_config.clone(),
                };

                info!("Adding model to registry: {}", name);
                configs.insert(name.to_string(), config);
                info!("Loaded configuration for model: {}", name);
            }
        }

        info!("Loaded {} model configurations", configs.len());
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
