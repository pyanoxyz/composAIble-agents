use crate::llm::llm_builder::LLM;
use std::error::Error as StdError;
use std::pin::Pin;
use super::agent_trait::AgentTrait;
use tokio_stream::StreamExt;
use crate::tools::Tool;
use std::sync::Arc;
use log::info;

pub struct Agent {
    pub(crate) system_prompt: Option<String>,
    pub(crate) user_prompt: Option<String>,
    pub(crate) stream: Option<bool>,
    pub(crate) llm: Option<LLM>,
    pub(crate) name: Option<String>,
    pub(crate) tools: Option<Vec<Arc<dyn Tool>>>, // New field for array of AgentTrait objects
}

impl AgentTrait for Agent {
    fn system_prompt(&self) -> Option<&String> {
        self.system_prompt.as_ref()
    }

    fn user_prompt(&self) -> Option<&String> {
        self.user_prompt.as_ref()
    }

    fn set_user_prompt(&mut self, prompt: String) {
        self.user_prompt = Some(prompt);
    }

    fn stream(&self) -> bool {
        self.stream.unwrap_or(false)
    }

    fn llm(&self) -> Option<&LLM> {
        self.llm.as_ref()
    }

    fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    fn invoke(
        &self
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<String, Box<dyn StdError + Send + Sync>>> +
                Send +
                '_
        >
    > {
        Box::pin(async move {
            let llm = self.llm.as_ref().expect("LLM is required");
            let system_prompt = self.system_prompt.as_ref().expect("System prompt is missing");
            let user_prompt = self.user_prompt.as_ref().expect("User prompt is missing");
            let stream = self.stream.unwrap_or(false);

            let mut output = String::new(); // Buffer to collect the streamed output

            if stream {
                let mut response_stream = llm.response_stream(user_prompt, system_prompt).await?;
                while let Some(response) = response_stream.next().await {
                    match response {
                        Ok(bytes) => {
                            let chunk = String::from_utf8_lossy(&bytes).to_string();
                            print!("{}", chunk); // Stream to the console
                            output.push_str(&chunk); // Collect into buffer
                        }
                        Err(e) => eprintln!("Error streaming response: {}", e),
                    }
                }
            } else {
                let response = llm.response(user_prompt, system_prompt).await?;
                // Safely extract and convert `response["content"]` to a string
                if let Some(content) = response.get("content").and_then(|v| v.as_str()) {
                    println!("Response: {}", content);
                    output.push_str(content); // Append content to output buffer
                } else {
                    eprintln!("Error: `content` field is missing or not a string in the response");
                }
            }

            Ok(output) // Return the complete output
        })
    }

    /// Generates a formatted string representation of all tools available in the Agent.
    ///
    /// This function iterates over the `tools` associated with the `Agent`,
    /// retrieves each tool's `name`, `description`, and `parameters` using
    /// their respective trait methods, and formats the information into
    /// JSON-like strings. The formatted strings are then wrapped between
    /// `<tools>` and `</tools>` tags for a structured output.
    ///
    /// # Returns
    /// A string representation of the tools, formatted as:
    /// ```
    /// <tools>
    /// {"name":"tool_name_1","description":"Description of tool 1.","parameters":{...}}
    /// {"name":"tool_name_2","description":"Description of tool 2.","parameters":{...}}
    /// </tools>
    /// ```
    ///
    /// If no tools are available, the output will be:
    /// ```
    /// <tools>
    /// </tools>
    /// ```
    fn get_tools(&self) -> String {
        if let Some(tools) = &self.tools {
            // Iterate through the tools, calling the description method of each tool
            let tool_descriptions: Vec<String> = tools
                .iter()
                .map(|tool| {
                    format!(
                        r#"{{"name":"{}","description":"{}","parameters":{}}}"#,
                        tool.name(),
                        tool.description(),
                        tool.parameters()
                    )
                })
                .collect();
            // Wrap the tools in <tools> and </tools> tags
            format!("<tools>\n{}\n</tools>", tool_descriptions.join("\n"))
        } else {
            // Return empty tools format if there are no tools
            "<tools>\n</tools>".to_string()
        }
    }
}
