use async_trait::async_trait;
use serde_json::json;
use reqwest::Client;

use super::manager_trait::ModelManagerInterface;
use super::types::{ ModelConfig, ModelInfo, ModelStatus };
use super::error::ModelResult;
use crate::llm::llm_builder::{ LLMBuilder, LLM };
use super::state::ModelState;
use crate::llm::options::LLMHTTPCallOptions;
use crate::llm::stream_processing::{ llamacpp_process_stream, qwen_process_stream };

pub struct ModelManagerClient {
    base_url: String,
    client: Client,
}

impl ModelManagerClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl ModelManagerInterface for ModelManagerClient {
    async fn load_model(&self, _state: ModelState) -> ModelResult<()> {
        let _url = format!("{}/models/load", self.base_url);
        // let _ = self.client.post(&url).json(&config).send().await?.error_for_status();
        Ok(())
    }

    async fn load_model_by_name(&self, _name: &str) -> ModelResult<()> {
        // todo
        Ok(())
    }

    async fn unload_model(&self, name: &str) -> ModelResult<()> {
        let url = format!("{}/models/unload", self.base_url);
        let _ = self.client
            .post(&url)
            .json(&json!({ "name": name }))
            .send().await?
            .error_for_status();
        Ok(())
    }

    async fn get_model_status(&self, name: &str) -> ModelResult<ModelStatus> {
        let url = format!("{}/models/status/{}", self.base_url, name);
        let response = self.client.get(&url).send().await?.error_for_status()?;

        Ok(response.json().await?)
    }

    async fn list_models(&self) -> ModelResult<Vec<ModelInfo>> {
        let url = format!("{}/models/list", self.base_url);
        let response = self.client.get(&url).send().await?.error_for_status()?;

        Ok(response.json().await?)
    }

    async fn get_llm(
        &self,
        model_name: &str,
        options: Option<LLMHTTPCallOptions>
    ) -> ModelResult<LLM> {
        // todo add this to server
        // First ensure the model is loaded
        match self.get_model_status(model_name).await {
            Ok(_) => (), // Model is already loaded
            Err(_) => {
                // If model is not found/loaded, get its config and load it
                let url = format!("{}/models/config/{}", self.base_url, model_name);
                let config: ModelConfig = self.client
                    .get(&url)
                    .send().await?
                    .error_for_status()?
                    .json().await?;

                self.load_model(ModelState::new(config)).await?;
            }
        }
        // Get the model's server details
        let url = format!("{}/models/server/{}", self.base_url, model_name);
        let server_info: serde_json::Value = self.client
            .get(&url)
            .send().await?
            .error_for_status()?
            .json().await?;

        // Create LLM with the server information
        let llm_options = options.unwrap_or_default();
        llm_options
            .with_server_url(
                format!(
                    "http://{}:{}",
                    server_info["host"].as_str().unwrap_or("localhost"),
                    server_info["port"].as_u64().unwrap_or(8000)
                )
            )
            .with_prompt_template(
                server_info["prompt_template"]
                    .as_str()
                    .unwrap_or("{system_prompt}\n{user_prompt}")
                    .to_string()
            );

        let _processor = match server_info["model_kind"].as_str() {
            Some("LLaMA") => llamacpp_process_stream,
            Some("Qwen") => qwen_process_stream,
            _ => llamacpp_process_stream, // default
        };

        Ok(LLMBuilder::default().build())
    }
}
