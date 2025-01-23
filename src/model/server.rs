use std::sync::Arc;

use axum::{
    routing::{ get, post },
    Router,
    Json,
    extract::State,
    response::IntoResponse,
    http::StatusCode,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use serde_json::json;
use crate::model::state::ModelState;

use crate::model::error::{ ModelError, ModelResult };
use super::{ ModelConfig, ModelManager };

pub struct ModelManagerServer {
    manager: Arc<ModelManager>,
}

impl ModelManagerServer {
    pub fn new(manager: Arc<ModelManager>) -> Self {
        Self { manager }
    }

    pub async fn run(self, addr: &str) -> ModelResult<()> {
        let app = Router::new()
            .route("/models/load", post(Self::handle_load_model))
            .route("/models/unload", post(Self::handle_unload_model))
            .route("/models/status/:name", get(Self::handle_get_status))
            // .route("/models/list", get(Self::handle_list_models))
            .with_state(self.manager);

        println!("Model Manager server starting on {}", addr);

        // Parse the address
        let addr: SocketAddr = addr
            .parse()
            .map_err(|e| ModelError::ConfigError(format!("Invalid address: {}", e)))?;

        // Create the listener
        let listener = TcpListener::bind(addr).await.map_err(|e| ModelError::IoError(e))?;

        // Start the server
        axum::serve(listener, app).await.map_err(|e| ModelError::IoError(e))?;

        Ok(())
    }

    async fn handle_load_model(
        State(manager): State<Arc<ModelManager>>,
        Json(config): Json<ModelConfig>
    ) -> impl IntoResponse {
        match manager.load_model(ModelState::new(config)).await {
            Ok(()) => (StatusCode::OK, Json(())).into_response(),
            Err(e) =>
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": e.to_string() })),
                ).into_response(),
        }
    }

    async fn handle_unload_model(
        State(manager): State<Arc<ModelManager>>,
        Json(name): Json<String>
    ) -> impl IntoResponse {
        match manager.unload_model(&name).await {
            Ok(()) => (StatusCode::OK, Json(())).into_response(),
            Err(e) =>
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": e.to_string() })),
                ).into_response(),
        }
    }

    async fn handle_get_status(
        State(manager): State<Arc<ModelManager>>,
        axum::extract::Path(name): axum::extract::Path<String>
    ) -> impl IntoResponse {
        match manager.get_model_status(&name).await {
            Ok(status) => (StatusCode::OK, Json(status)).into_response(),
            Err(e) =>
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": e.to_string() })),
                ).into_response(),
        }
    }

    // async fn handle_list_models(State(manager): State<Arc<ModelManager>>) -> impl IntoResponse {
    //     match manager.list_models().await {
    //         Ok(models) => (StatusCode::OK, Json(models)).into_response(),
    //         Err(e) =>
    //             (
    //                 StatusCode::INTERNAL_SERVER_ERROR,
    //                 Json(json!({ "error": e.to_string() })),
    //             ).into_response(),
    //     }
    // }
}
