use std::path::PathBuf;
use std::process::Command;
use super::super::utils::get_env_var;
use super::super::state::ModelState;

#[derive(Debug)]
pub(crate) struct LlamaProcess {
    pub state: ModelState,
    pub cmd: Option<Command>,
}

impl LlamaProcess {
    pub fn new(state: ModelState) -> Self {
        Self {
            state: state,
            cmd: None,
        }
    }

    pub async fn getcmd(&mut self) {
        /* ToDO Implement server based on machine type */
        let adapters_dir = get_env_var("ADAPTERS_HOME").unwrap_or(
            "pyano_home/adapters".to_string()
        );

        let cmd_path = (
            if cfg!(target_os = "macos") {
                if cfg!(target_arch = "aarch64") {
                    format!("{}/llama/macos/arm64/llama-server", adapters_dir)
                } else {
                    format!("{}/llama/macos/x64/llama-server", adapters_dir)
                }
            } else {
                format!("{}/llama/ubuntu/llama-server", adapters_dir)
            }
        ).to_string();

        let mut cmd = Command::new(&cmd_path);

        let model_path: PathBuf = get_env_var("MODEL_HOME")
            .map(|path| PathBuf::from(path))
            .expect("MODEL_HOME environment variable not set");

        let model_path = model_path.join(&*self.state.model_path.lock().unwrap());

        // Configure command based on adapter config
        cmd.arg("-m")
            .arg(&model_path)
            .arg("--ctx-size")
            .arg(self.state.config.server_config.ctx_size.to_string());

        if let Some(port) = *self.state.port.lock().unwrap() {
            cmd.arg("--port").arg(port.to_string());
        }

        if let Some(threads) = self.state.config.server_config.num_threads {
            cmd.arg("--threads").arg(threads.to_string());
        }

        if self.state.config.server_config.gpu_layers > 0 {
            cmd.arg("--n-gpu-layers").arg(self.state.config.server_config.gpu_layers.to_string());
        }

        if !self.state.config.server_config.use_mmap {
            cmd.arg("--no-mmap");
        }

        // Add batch size
        cmd.arg("--batch-size").arg(self.state.config.server_config.batch_size.to_string());

        // Add extra arguments
        for (key, value) in &self.state.config.server_config.extra_args {
            cmd.arg(format!("--{}", key)).arg(value);
        }

        self.cmd = Some(cmd);
    }
}
