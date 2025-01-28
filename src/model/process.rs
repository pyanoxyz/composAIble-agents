use std::process::Child;
use std::thread::sleep;
use std::time::Duration;
use chrono::Utc;
use tokio::sync::oneshot;
use log::{ info, error };
use super::adapters::llama::LlamaProcess;
use super::{ ModelConfig, ModelStatus };
use super::state::ModelState;
use super::error::{ ModelError, ModelResult };
pub(crate) struct ModelProcess {
    pub state: ModelState,
    pub child: Option<Child>,
    pub shutdown_signal: Option<oneshot::Sender<()>>,
    pub model_process: Option<Box<LlamaProcess>>,
}

impl ModelProcess {
    pub fn new(state: ModelState) -> Self {
        Self {
            state,
            child: None,
            shutdown_signal: None,
            model_process: None,
        }
    }

    pub async fn start(&mut self) -> ModelResult<()> {
        self.state.show_state();
        if *self.state.status.lock().unwrap() == ModelStatus::Running {
            return Ok(());
        }
        info!("Starting model {}", self.state.config.model_config.name);
        self.state.show_state();
        *self.state.status.lock().unwrap() = ModelStatus::Loading;
        self.model_process = Some(Box::new(LlamaProcess::new(self.state.clone())));
        self.model_process.as_mut().unwrap().getcmd().await;
        let cmd = self.model_process.as_mut().unwrap().cmd.as_mut().unwrap();
        info!("Starting model with command: {:?}", cmd);
        match self.model_process.as_mut().unwrap().cmd.as_mut().unwrap().spawn() {
            Ok(child) => {
                sleep(Duration::from_secs(10));
                self.child = Some(child);
                *self.state.status.lock().unwrap() = ModelStatus::Running;
                *self.state.last_used.lock().unwrap() = Utc::now();
                Ok(())
            }
            Err(e) => {
                *self.state.status.lock().unwrap() = ModelStatus::Error(e.to_string());
                Err(ModelError::ProcessError(e.to_string()))
            }
        }
    }
    pub async fn stop(&mut self) -> ModelResult<()> {
        if let Some(mut child) = self.child.take() {
            let pid = child.id();

            // Try graceful shutdown first
            if let Err(e) = child.kill() {
                error!("Failed to kill process gracefully: {}", e);
                // Force kill as backup
                unsafe {
                    libc::kill(pid as i32, libc::SIGKILL);
                }
            }
            sleep(Duration::from_secs(5));
            // Wait for process to exit with timeout
            let _ = tokio::time::timeout(std::time::Duration::from_secs(1), async {
                let mut child = child;
                let _ = child.wait();
            }).await;

            // Force kill again if still running
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
            }
        }

        *self.state.status.lock().unwrap() = ModelStatus::Stopped;
        self.child = None;

        Ok(())
    }
}
