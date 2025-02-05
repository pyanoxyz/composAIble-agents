// This code is adapted or copied from another location.
// Original source: [Abraxas-365/langchain-rustsrc/src/tools/duckduckgo/duckduckgo_search.rs].
// Ensure that the usage complies with the original license terms, if applicable.

use std::{ collections::HashMap, error::Error };

use async_trait::async_trait;
use reqwest::Client;
use scraper::{ Html, Selector };
use serde::{ Deserialize, Serialize };
use serde_json::{ json, Value };
use url::Url;
use log::{ info, error };

use crate::tools::Tool;

pub struct DuckDuckGoSearchResults {
    url: String,
    client: Client,
    max_results: usize,
}

impl DuckDuckGoSearchResults {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            url: "https://duckduckgo.com/html/".to_string(),
            max_results: 4,
        }
    }

    pub fn with_max_results(mut self, max_results: usize) -> Self {
        self.max_results = max_results;
        self
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let mut url = Url::parse(&self.url)?;

        let mut query_params = HashMap::new();
        query_params.insert("q", query);

        url.query_pairs_mut().extend_pairs(query_params.iter());
        info!("Query URL: {}", url);

        let response = self.client.get(url).send().await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);

        let result_selector = Selector::parse(".web-result").unwrap();
        let result_title_selector = Selector::parse(".result__a").unwrap();
        let result_url_selector = Selector::parse(".result__url").unwrap();
        let result_snippet_selector = Selector::parse(".result__snippet").unwrap();

        let results = document
            .select(&result_selector)
            .map(|result| {
                let title = result
                    .select(&result_title_selector)
                    .next()
                    .map_or(String::new(), |el| el.text().collect::<Vec<_>>().join(""));
                let link = result
                    .select(&result_url_selector)
                    .next()
                    .map_or(String::new(), |el|
                        el.text().collect::<Vec<_>>().join("").trim().to_string()
                    );
                let snippet = result
                    .select(&result_snippet_selector)
                    .next()
                    .map_or(String::new(), |el| el.text().collect::<Vec<_>>().join(""));

                SearchResult {
                    title,
                    link,
                    snippet,
                }
            })
            .take(self.max_results)
            .collect::<Vec<_>>();

        Ok(results)
    }

    pub fn extract_links_from_results(response: Value) -> Vec<String> {
        if let Some(results) = response["results"].as_array() {
            results
                .iter()
                .filter_map(|result| result["link"].as_str().map(|s| s.to_string()))
                .collect()
        } else {
            // Log an error or handle cases where "results" is not present or not an array
            error!("No valid results found in the response.");
            vec![]
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    title: String,
    link: String,
    snippet: String,
}

#[async_trait]
impl Tool for DuckDuckGoSearchResults {
    fn name(&self) -> String {
        String::from("DuckDuckGoSearch")
    }

    fn description(&self) -> String {
        String::from(
            r#""Wrapper for DuckDuckGo Search API. "
	"Useful for when you need to answer questions about current events. "
	"Always one of the first options when you need to find information on internet"
	"Input should be a search query. Output is a JSON array of the query results."#
        )
    }

    // async fn run(&self, input: Value) -> Result<Value, Box<dyn Error>> {
    //     let query = input["query"].as_str().ok_or("Input should be a string in the 'query' field")?;
    //     info!("Searching [{}] on DuckDuckGo", query);

    //     let results = self.search(query).await?;
    //     Ok(serde_json::to_value(results)?)
    // }

    async fn run(&self, input: Value) -> Result<Value, Box<dyn Error>> {
        // Extract the query string from the input
        let query = input["query"]
            .as_str()
            .ok_or("Input must be a JSON object with a 'query' field of type string")?;

        // Call the `search` function and handle its result
        match self.search(query).await {
            Ok(results) => {
                // Convert the results to a JSON value
                let json_results = serde_json::to_value(results)?;
                Ok(
                    json!({
                    "query": query,
                    "results": json_results
                })
                )
            }
            Err(e) => {
                // Return an error as a JSON object
                Ok(
                    json!({
                    "query": query,
                    "error": format!("Error performing search: {}", e)
                })
                )
            }
        }
    }

    fn parameters(&self) -> Value {
        let prompt =
            r#"A wrapper around DuckDuckGo Search.
            Useful for when you need to answer questions about current events.
            Input should be a search query. Output is a JSON array of the query results."#;

        json!({
            "description": prompt,
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query to look up"
                }
            },
            "required": ["query"]
        })
    }
}

impl Default for DuckDuckGoSearchResults {
    fn default() -> DuckDuckGoSearchResults {
        DuckDuckGoSearchResults::new()
    }
}

#[cfg(test)]
mod tests {
    use super::DuckDuckGoSearchResults;

    #[tokio::test]
    #[ignore]
    async fn duckduckgosearch_tool() {
        let ddg = DuckDuckGoSearchResults::default().with_max_results(5);
        let s = ddg.search("Who is the current President of Peru?").await.unwrap();

        println!("{:?}", s);
    }
}
