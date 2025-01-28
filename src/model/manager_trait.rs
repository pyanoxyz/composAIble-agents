use async_trait::async_trait;
use crate::llm::llm_builder::LLM;
use super::state::ModelState;
use crate::llm::options::LLMHTTPCallOptions;
use super::types::{ ModelInfo, ModelStatus };
use super::error::ModelResult;

#[async_trait]
pub trait ModelManagerInterface: Send + Sync {
    async fn load_model(&self, state: ModelState) -> ModelResult<()>;
    async fn unload_model(&self, name: &str) -> ModelResult<()>;
    async fn get_model_status(&self, name: &str) -> ModelResult<ModelStatus>;
    async fn list_models(&self) -> ModelResult<Vec<ModelInfo>>;
    async fn get_llm(
        &self,
        model_name: &str,
        options: Option<LLMHTTPCallOptions>
    ) -> ModelResult<LLM>;
    async fn load_model_by_name(&self, name: &str) -> ModelResult<()>;
}
