use std::thread::sleep;
use std::time::Duration;
use chrono::Utc;
use tokio::sync::oneshot;
use log::{ debug, error, info };
use super::adapters::llama::LlamaProcess;
use super::ModelStatus;
use super::state::ModelState;
use reqwest::Client;
use super::error::{ ModelError, ModelResult };
use std::process::{ Child, Stdio };
// use std::io::{ BufReader, BufRead };
// use std::thread;

const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(2);
const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(60);

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
    async fn check_health(&mut self, port: u16) -> bool {
        let client = Client::new();
        let url = format!("http://localhost:{}/health", port);

        // Check if process is still running
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(status)) => {
                    error!("Process exited with status: {}", status);
                    return false;
                }
                Err(e) => {
                    error!("Error checking process status: {}", e);
                    return false;
                }
                Ok(None) => {} // Process still running
            }
        }

        match client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    async fn wait_for_health_check(&mut self, port: u16) -> ModelResult<()> {
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < HEALTH_CHECK_TIMEOUT {
            if self.check_health(port).await {
                info!("Health check passed for model {}", self.state.config.model_config.name);
                return Ok(());
            }

            sleep(HEALTH_CHECK_INTERVAL);
        }

        Err(
            ModelError::ProcessError(
                format!(
                    "Health check timeout after {} seconds for model {}",
                    HEALTH_CHECK_TIMEOUT.as_secs(),
                    self.state.config.model_config.name
                )
            )
        )
    }
    pub async fn start(&mut self) -> ModelResult<()> {
        if *self.state.status.lock().unwrap() == ModelStatus::Running {
            return Ok(());
        }
        info!("Starting model {}", self.state.config.model_config.name);
        *self.state.status.lock().unwrap() = ModelStatus::Loading;
        self.model_process = Some(Box::new(LlamaProcess::new(self.state.clone())));
        self.model_process.as_mut().unwrap().getcmd().await;

        let cmd = self.model_process.as_mut().unwrap().cmd.as_mut().unwrap();

        // Configure the command to pipe stdout and stderr
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        debug!("Starting model with command: {:?}", cmd);
        match self.model_process.as_mut().unwrap().cmd.as_mut().unwrap().spawn() {
            Ok(child) => {
                // Capture stdout in a separate thread
                // if let Some(stdout) = child.stdout.take() {
                //     thread::spawn(move || {
                //         let reader = BufReader::new(stdout);
                //         for line in reader.lines() {
                //             if let Ok(line) = line {
                //                 // Log the output instead of printing to stdout
                //                 // You can also store it in a buffer if needed
                //                 info!("Model output: {}", line);
                //             }
                //         }
                //     });
                // }

                // Capture stderr in a separate thread
                // if let Some(stderr) = child.stderr.take() {
                //     thread::spawn(move || {
                //         let reader = BufReader::new(stderr);
                //         for line in reader.lines() {
                //             if let Ok(line) = line {
                //                 // Log the error output
                //                 error!("Model error: {}", line);
                //             }
                //         }
                //     });
                // }
                self.child = Some(child);

                // Get port from state or configuration

                let port = match *self.state.port.lock().unwrap() {
                    Some(port) => port,
                    None => {
                        return Err(ModelError::ProcessError("Port not configured".to_string()));
                    }
                };

                // Wait for health check to pass
                match self.wait_for_health_check(port).await {
                    Ok(()) => {
                        *self.state.status.lock().unwrap() = ModelStatus::Running;
                        *self.state.last_used.lock().unwrap() = Utc::now();
                    }
                    Err(e) => {
                        // Clean up the process if health check fails
                        if let Err(stop_err) = self.stop().await {
                            error!("Failed to stop process after health check failure: {}", stop_err);
                        }
                        return Err(ModelError::ProcessError(e.to_string()));
                    }
                }

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
