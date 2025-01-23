pub struct LLMServerOptions {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stop_words: Option<Vec<String>>,
    pub top_k: Option<usize>,
    pub top_p: Option<f32>,
    pub seed: Option<usize>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub repetition_penalty: Option<f32>,
}

#[derive(Clone)]
pub struct LLMHTTPCallOptions {
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub stop_words: Option<Vec<String>>,
    pub top_k: Option<usize>,
    pub top_p: Option<f32>,
    pub seed: Option<usize>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub repetition_penalty: Option<f32>,
    pub server_url: Option<String>,
    pub prompt_template: Option<String>,
    pub port: Option<u16>,
    initialized_fields: Vec<String>,
}

impl Default for LLMHTTPCallOptions {
    fn default() -> Self {
        LLMHTTPCallOptions {
            max_tokens: None,
            temperature: Some(0.4),
            stop_words: None,
            top_k: None,
            top_p: None,
            seed: None,
            min_length: None,
            max_length: None,
            repetition_penalty: None,
            server_url: None,
            prompt_template: None,
            port: None,
            initialized_fields: Vec::new(),
        }
    }
}

impl LLMHTTPCallOptions {
    pub fn new() -> Self {
        LLMHTTPCallOptions::default()
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self.initialized_fields.push("max_tokens".to_string());
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self.initialized_fields.push("temperature".to_string());
        self
    }

    pub fn with_stop_words(mut self, stop_words: Vec<String>) -> Self {
        self.stop_words = Some(stop_words);
        self.initialized_fields.push("stop_words".to_string());
        self
    }

    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = Some(top_k);
        self.initialized_fields.push("top_k".to_string());
        self
    }

    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self.initialized_fields.push("top_p".to_string());
        self
    }

    pub fn with_seed(mut self, seed: usize) -> Self {
        self.seed = Some(seed);
        self.initialized_fields.push("seed".to_string());
        self
    }

    pub fn with_min_length(mut self, min_length: usize) -> Self {
        self.min_length = Some(min_length);
        self.initialized_fields.push("min_length".to_string());
        self
    }

    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self.initialized_fields.push("max_length".to_string());
        self
    }

    pub fn with_repetition_penalty(mut self, repetition_penalty: f32) -> Self {
        self.repetition_penalty = Some(repetition_penalty);
        self.initialized_fields.push("repetition_penalty".to_string());
        self
    }

    pub fn with_server_url(mut self, server_url: String) -> Self {
        self.server_url = Some(server_url);
        self.initialized_fields.push("server_url".to_string());
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        let server_url = format!("http://localhost:{}", port);
        self.server_url = Some(server_url);
        self.port = Some(port);
        self.initialized_fields.push("server_url".to_string());
        self
    }

    pub fn with_prompt_template(mut self, prompt_template: String) -> Self {
        self.prompt_template = Some(prompt_template);
        self.initialized_fields.push("prompt_template".to_string());
        self
    }

    pub fn build(mut self) -> Self {
        // Initialize only fields that have been explicitly set
        let defaults = LLMHTTPCallOptions::default();

        if !self.initialized_fields.contains(&"max_tokens".to_string()) {
            self.max_tokens = defaults.max_tokens;
        }
        if !self.initialized_fields.contains(&"temperature".to_string()) {
            self.temperature = defaults.temperature;
        }
        if !self.initialized_fields.contains(&"stop_words".to_string()) {
            self.stop_words = defaults.stop_words;
        }
        if !self.initialized_fields.contains(&"top_k".to_string()) {
            self.top_k = defaults.top_k;
        }
        if !self.initialized_fields.contains(&"top_p".to_string()) {
            self.top_p = defaults.top_p;
        }
        if !self.initialized_fields.contains(&"seed".to_string()) {
            self.seed = defaults.seed;
        }
        if !self.initialized_fields.contains(&"min_length".to_string()) {
            self.min_length = defaults.min_length;
        }
        if !self.initialized_fields.contains(&"max_length".to_string()) {
            self.max_length = defaults.max_length;
        }
        if !self.initialized_fields.contains(&"repetition_penalty".to_string()) {
            self.repetition_penalty = defaults.repetition_penalty;
        }

        if
            !self.initialized_fields.contains(&"server_url".to_string()) &&
            !self.initialized_fields.contains(&"port".to_string())
        {
            panic!("server_url or port must be provided before calling build()");
        }

        if !self.initialized_fields.contains(&"prompt_template".to_string()) {
            panic!("prompt_template must be provided before calling build()");
        }

        self
    }
}
