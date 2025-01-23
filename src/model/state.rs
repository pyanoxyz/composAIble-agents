use std::sync::{ Arc, Mutex };
use chrono::{ DateTime, Utc };
use crate::model::{ ModelConfig, ModelStatus };
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ModelState {
    // Configuration
    pub config: ModelConfig,

    // Model Confguratons
    pub model_path: Arc<Mutex<PathBuf>>,
    pub model_url: Arc<Mutex<std::option::Option<std::string::String>>>,

    //Dyanmic Memory Configurations (Only to be changed when n advance user settng)
    pub min_ram_usage: Arc<Mutex<f32>>,
    pub recommended_ram_gb: Arc<Mutex<f32>>,
    pub gpu_memory_gb: Arc<Mutex<Option<f32>>>,

    // Model Parameters
    pub temperature: Arc<Mutex<f32>>,
    pub top_k: Arc<Mutex<usize>>,
    pub top_p: Arc<Mutex<f32>>,
    pub max_tokens: Arc<Mutex<usize>>,
    pub repetition_penalty: Arc<Mutex<f32>>,

    // Runtime state
    pub status: Arc<Mutex<ModelStatus>>,
    pub last_used: Arc<Mutex<DateTime<Utc>>>,
    pub port: Arc<Mutex<Option<u16>>>,
    pub server_url: Arc<Mutex<Option<String>>>,

    // Process management
    pub process_id: Arc<Mutex<Option<u32>>>,
}

impl Default for ModelState {
    // implement default please
    fn default() -> Self {
        Self {
            config: ModelConfig::default(),
            model_path: Arc::new(Mutex::new(PathBuf::new())),
            model_url: Arc::new(Mutex::new(None)),
            min_ram_usage: Arc::new(Mutex::new(0.0)),
            recommended_ram_gb: Arc::new(Mutex::new(0.0)),
            gpu_memory_gb: Arc::new(Mutex::new(None)),
            temperature: Arc::new(Mutex::new(0.0)),
            top_k: Arc::new(Mutex::new(0)),
            top_p: Arc::new(Mutex::new(0.0)),
            max_tokens: Arc::new(Mutex::new(0)),
            repetition_penalty: Arc::new(Mutex::new(0.0)),
            status: Arc::new(Mutex::new(ModelStatus::Stopped)),
            last_used: Arc::new(Mutex::new(Utc::now())),
            port: Arc::new(Mutex::new(None)),
            server_url: Arc::new(Mutex::new(None)),
            process_id: Arc::new(Mutex::new(None)),
        }
    }
}

impl ModelState {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            config: config.clone(),
            model_path: Arc::new(Mutex::new(config.model_config.model_path.clone())),
            model_url: Arc::new(Mutex::new(config.model_config.model_url.clone())),
            min_ram_usage: Arc::new(Mutex::new(config.memory_config.min_ram_gb)),
            recommended_ram_gb: Arc::new(Mutex::new(config.memory_config.recommended_ram_gb)),
            gpu_memory_gb: Arc::new(Mutex::new(config.memory_config.gpu_memory_gb)),
            temperature: Arc::new(Mutex::new(config.defaults.temperature)),
            top_k: Arc::new(Mutex::new(config.defaults.top_k)),
            top_p: Arc::new(Mutex::new(config.defaults.top_p)),
            max_tokens: Arc::new(Mutex::new(config.defaults.max_tokens)),
            repetition_penalty: Arc::new(Mutex::new(config.defaults.repetition_penalty)),
            port: Arc::new(Mutex::new(config.server_config.port)),
            server_url: Arc::new(
                Mutex::new(
                    Some(format!("http://localhost:{}", config.server_config.port.unwrap_or(0)))
                )
            ),
            status: Arc::new(Mutex::new(ModelStatus::Stopped)),
            last_used: Arc::new(Mutex::new(Utc::now())),
            process_id: Arc::new(Mutex::new(None)),
        }
    }

    pub fn update_status(&self, new_status: ModelStatus) {
        let mut status = self.status.lock().unwrap();
        *status = new_status;
    }
    pub fn update_last_used(&self, last_used: DateTime<Utc>) {
        let mut last_used_guard = self.last_used.lock().unwrap();
        *last_used_guard = last_used;
    }
    pub fn update_process_id(&self, process_id: u32) {
        let mut process_id_guard = self.process_id.lock().unwrap();
        *process_id_guard = Some(process_id);
    }
    pub fn update_server_url(&self, server_url: String) {
        let mut server_url_guard = self.server_url.lock().unwrap();
        *server_url_guard = Some(server_url);
    }
    pub fn show_state(&self) {
        let status = self.status.lock().unwrap();
        //print all the variables that show information about the model,
        //like model_path, model_url, min_ram_usage, recommended_ram_gb, gpu_memory_gb, temperature, top_k, top_p, max_tokens, repetition_penalty, port, status, last_used, process_id
        println!("Model Path: {:?}", self.model_path.lock().unwrap());
        println!("Temperature: {:?}", self.temperature.lock().unwrap());
        println!("Top K: {:?}", self.top_k.lock().unwrap());
        println!("Top P: {:?}", self.top_p.lock().unwrap());
        println!("Max Tokens: {:?}", self.max_tokens.lock().unwrap());
        println!("Repetition Penalty: {:?}", self.repetition_penalty.lock().unwrap());
        println!("Port: {:?}", self.port.lock().unwrap());
        println!("Status: {:?}", status);
        println!("Last Used: {:?}", self.last_used.lock().unwrap());
        println!("Process ID: {:?}", self.process_id.lock().unwrap());
    }
}
