use std::error::Error as StdError;
use log::{ info, error };

use crate::agent::agent_trait::AgentTrait;
use std::sync::{ Arc, Mutex };

#[derive(Clone)]
pub struct ExecutionRecord {
    pub agent_name: String,
    pub input: String,
    pub output: String,
    pub timestamp: std::time::SystemTime,
}

pub trait ExecutionRecorder: Send + Sync {
    fn store_execution(
        &self,
        agent_name: &str,
        input: &str,
        output: &str
    ) -> Result<(), Box<dyn StdError + Send + Sync>>;
}

pub struct Chain {
    agents: Vec<Arc<Mutex<dyn AgentTrait>>>, // Use Arc<Mutex> for mutable trait objects
    recorder: Option<Arc<dyn ExecutionRecorder>>,
    memory_log: Arc<Mutex<Vec<ExecutionRecord>>>,
}

impl Chain {
    pub fn new() -> Self {
        Chain {
            agents: Vec::new(),
            recorder: None,
            memory_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_recorder(mut self, recorder: Arc<dyn ExecutionRecorder>) -> Self {
        self.recorder = Some(recorder);
        self
    }

    pub fn add_agent(mut self, agent: Arc<Mutex<dyn AgentTrait>>) -> Self {
        self.agents.push(agent);
        info!("Added agent");
        self
    }

    /// Run all agents in sequence.
    /// The output of agent i is passed as user_prompt to agent i+1.
    pub async fn run(&mut self) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let mut previous_output: Option<String> = None;

        for agent in &self.agents {
            let mut agent = agent.lock().unwrap();
            info!("EXECUTING Agent = {:?}", agent.name());

            // If we have a previous output, set it as the user prompt for the current agent.
            if let Some(output) = &previous_output {
                agent.set_user_prompt(output.clone());
            }

            // Get the current user prompt and execute the agent
            let user_input = agent.user_prompt().cloned().unwrap_or_default();
            info!("agent invoked");
            let output = match agent.invoke().await {
                Ok(output) => {
                    // Process output
                    previous_output = Some(output.clone());
                    output
                }
                Err(e) => {
                    error!("Agent {} failed: {}", agent.name().cloned().unwrap_or_default(), e);
                    return Err(e);
                }
            };

            // Store in memory log
            {
                let mut log = self.memory_log.lock().unwrap();
                log.push(ExecutionRecord {
                    agent_name: agent
                        .name()
                        .cloned()
                        .unwrap_or_else(|| "Unnamed Agent".to_string()),
                    input: user_input.clone(),
                    output: output.clone(),
                    timestamp: std::time::SystemTime::now(),
                });
            }

            // Store in recorder (if any)
            if let Some(recorder) = &self.recorder {
                recorder.store_execution(
                    agent.name().unwrap_or(&"Unnamed Agent".to_string()),
                    &user_input,
                    &output
                )?;
            }

            // Pass the output to the next agent
            previous_output = Some(output);
        }

        Ok(())
    }

    /// Access the memory logs
    pub fn memory_logs(&self) -> Vec<ExecutionRecord> {
        self.memory_log.lock().unwrap().clone()
    }
}
