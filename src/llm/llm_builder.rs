use crate::model::error::ModelError;
use crate::model::{ ModelManagerInterface, ModelStatus };
use log::error;
use super::{ options::LLMHTTPCallOptions, error::LLMError };
use std::error::Error as StdError; // Importing the correct trait
use std::pin::Pin;
use bytes::Bytes;
use futures::Stream;
use log::info; // Ensure StreamExt is imported
use std::sync::Arc;
use crate::model::state::ModelState;

#[derive(Clone)]
pub struct LLM {
    state: ModelState,
    client: reqwest::Client,
    options: LLMHTTPCallOptions,
    process_response: Option<
        Arc<
            dyn (Fn(
                Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>
            ) -> Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>) +
                Send +
                Sync
        >
    >,
    model_manager: Option<Arc<dyn ModelManagerInterface>>,
    model_name: Option<String>,
    auto_load: bool,
}

impl LLM {
    pub fn builder() -> LLMBuilder {
        LLMBuilder::default()
    }

    pub async fn load(self) {
        let manager = self.model_manager.unwrap();
        // Load the model
        let model_status = manager.get_model_status(&self.state.config.model_config.name).await;
        match model_status {
            Ok(ModelStatus::Running) => {
                info!("Model {} is already running", self.state.config.model_config.name);
            }
            _ => {
                info!("Loading model: {}", self.state.config.model_config.name);
                manager.load_model(self.state.clone()).await;
                // Verify model was loaded successfully
                match manager.get_model_status(&self.state.config.model_config.name).await.unwrap() {
                    ModelStatus::Running => {
                        info!("Model {} loaded successfully", self.state.config.model_config.name);
                    }
                    status => {
                        error!(
                            "Model {} failed to load properly",
                            self.state.config.model_config.name
                        );
                    }
                }
            }
        }
    }

    async fn prepare_request(
        &self,
        prompt_with_context: &str,
        system_prompt: &str,
        stream: bool
    ) -> Result<reqwest::Response, Box<dyn StdError + Send + Sync + 'static>> {
        let server_url = self.state.server_url.as_ref();

        let prompt_template = self.options.prompt_template
            .as_ref()
            .expect("Prompt template is missing");

        let full_prompt = prompt_template
            .replace("{system_prompt}", system_prompt)
            .replace("{user_prompt}", prompt_with_context);

        let mut json_payload = serde_json::Map::new();
        json_payload.insert("prompt".to_string(), serde_json::Value::String(full_prompt));
        json_payload.insert("stream".to_string(), serde_json::Value::Bool(stream));
        json_payload.insert("cache_prompt".to_string(), serde_json::Value::Bool(true));

        if let Some(temperature) = self.options.temperature {
            json_payload.insert(
                "temperature".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(temperature as f64).unwrap())
            );
        }
        if let Some(top_k) = self.options.top_k {
            json_payload.insert(
                "top_k".to_string(),
                serde_json::Value::Number(serde_json::Number::from(top_k as i64))
            );
        }
        if let Some(top_p) = self.options.top_p {
            json_payload.insert(
                "top_p".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(top_p as f64).unwrap())
            );
        }
        if let Some(seed) = self.options.seed {
            json_payload.insert(
                "seed".to_string(),
                serde_json::Value::Number(serde_json::Number::from(seed as i64))
            );
        }
        if let Some(min_length) = self.options.min_length {
            json_payload.insert(
                "min_length".to_string(),
                serde_json::Value::Number(serde_json::Number::from(min_length as i64))
            );
        }
        if let Some(max_length) = self.options.max_length {
            json_payload.insert(
                "max_length".to_string(),
                serde_json::Value::Number(serde_json::Number::from(max_length as i64))
            );
        }
        if let Some(repetition_penalty) = self.options.repetition_penalty {
            json_payload.insert(
                "repetition_penalty".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(repetition_penalty as f64).unwrap()
                )
            );
        }

        let resp = self.client
            .post(&format!("{}/completion", server_url.lock().unwrap().as_ref().unwrap()))
            .json(&serde_json::Value::Object(json_payload))
            .send().await
            .map_err(|e| LLMError::RequestFailed(e.to_string()))?
            .error_for_status()
            .map_err(|e| {
                if e.status().map_or(false, |status| status.is_server_error()) {
                    LLMError::ServerUnavailable(e.to_string())
                } else {
                    LLMError::RequestFailed(e.to_string())
                }
            })?;

        Ok(resp)
    }

    pub async fn response_stream(
        &self,
        prompt_with_context: &str,
        system_prompt: &str
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
        Box<dyn StdError + Send + Sync + 'static>
    > {
        info!("Response stream not wating");
        self.ensure_model_loaded().await?;

        let resp = self.prepare_request(prompt_with_context, system_prompt, true).await?;

        let stream = resp.bytes_stream();
        let processed_stream = if let Some(process_fn) = &self.process_response {
            process_fn(Box::pin(stream))
        } else {
            Box::pin(stream)
        };

        Ok(processed_stream)
    }

    pub async fn response(
        &self,
        prompt_with_context: &str,
        system_prompt: &str
    ) -> Result<serde_json::Value, Box<dyn StdError + Send + Sync + 'static>> {
        self.ensure_model_loaded().await?;

        let resp = self.prepare_request(prompt_with_context, system_prompt, false).await?;
        let response_json = resp.json::<serde_json::Value>().await?;
        Ok(response_json)
    }

    async fn ensure_model_loaded(&self) -> Result<(), Box<dyn StdError + Send + Sync>> {
        info!("Checking model status");
        if let (Some(manager), Some(name)) = (&self.model_manager, &self.model_name) {
            let should_load = match manager.get_model_status(name).await {
                Ok(ModelStatus::Running) => {
                    info!("Model {} is already running", name);
                    false
                }
                Ok(status) => {
                    info!("Model {} has status {:?}, will attempt to load", name, status);
                    true
                }
                Err(ModelError::ModelNotFound(_)) => {
                    info!("Model {} not found, will attempt to load", name);
                    true
                }
                Err(e) => {
                    info!("Unexpected error checking model status: {:?}", e);
                    return Err(Box::new(e));
                }
            };

            if should_load {
                info!("Loading model {}", name);
                manager.load_model_by_name(name).await?;

                // Verify the model loaded successfully
                (match manager.get_model_status(name).await {
                    Ok(ModelStatus::Running) => {
                        info!("Model {} loaded successfully", name);
                        Ok(())
                    }
                    Ok(status) => {
                        Err(
                            Box::new(
                                ModelError::ProcessError(
                                    format!("Failed to load model: {}. Status: {:?}", name, status)
                                )
                            )
                        )
                    }
                    Err(e) => Err(Box::new(e)),
                })?;
            }
        }

        Ok(())
    }
}

pub struct LLMBuilder {
    state: ModelState,
    options: LLMHTTPCallOptions,
    process_response: Option<
        Arc<
            dyn (Fn(
                Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>
            ) -> Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>) +
                Send +
                Sync
        >
    >,

    model_manager: Option<Arc<dyn ModelManagerInterface>>,
    model_name: Option<String>,
    auto_load: bool,
}

impl Default for LLMBuilder {
    fn default() -> Self {
        LLMBuilder {
            state: ModelState::default(),
            options: LLMHTTPCallOptions::new(),
            process_response: None, // Default to no custom processing
            auto_load: false,
            model_manager: None,
            model_name: None,
        }
    }
}

impl LLMBuilder {
    pub fn with_model_manager(
        mut self,
        manager: Arc<dyn ModelManagerInterface>,
        model_name: String,
        auto_load: bool
    ) -> Self {
        self.model_manager = Some(manager);
        self.model_name = Some(model_name);
        self.auto_load = auto_load;
        self
    }

    pub fn with_state(mut self, state: ModelState) -> Self {
        self.state = state;
        self
    }

    pub fn with_options(mut self, options: LLMHTTPCallOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_process_response<F>(mut self, process_fn: F) -> Self
        where
            F: Fn(
                Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>
            ) -> Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>> +
                Send +
                Sync +
                'static
    {
        self.process_response = Some(Arc::new(process_fn));
        self
    }

    pub fn build(self) -> LLM {
        LLM {
            state: self.state,
            client: reqwest::Client::new(),
            options: self.options.build(),
            process_response: self.process_response,
            model_manager: self.model_manager,
            model_name: self.model_name,
            auto_load: self.auto_load,
        }
    }
}
