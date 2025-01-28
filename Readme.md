# 🚀 Pyano Framework

A composable, resource-efficient framework for building AI applications locally in Rust. Inspired by langchain-rs[https://github.com/Abraxas-365/langchain-rust]

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🌟 Features

-   **📚 Local Model Management**: Advanced model lifecycle management with intelligent resource allocation
-   **🔄 Composable Agents**: Build complex AI workflows by chaining multiple agents together
-   **💾 Vector Storage**: Built-in support for efficient vector storage and similarity search using SQLite
-   **🔍 Embeddings**: Integrated text embedding capabilities using state-of-the-art models
-   **🛠 Flexible Tools**: Built-in tools for web scraping, search, and command execution
-   **🌐 Web Interface**: Clean HTTP interface for model management and inference
-   **🧠 Memory Efficient**: Smart memory management for running multiple models

## 🏗 Architecture

```
pyano-framework/
├── agent/         # Agent implementations and traits
├── chain/         # Sequential chain execution
├── embedding/     # Text embedding capabilities
├── llm/          # LLM interface and processing
├── model/        # Model management and server
├── tools/        # Utility tools (scraping, search)
├── vectorstore/  # Vector storage implementations
└── schemas/      # Common data structures
```

## 🚀 Quick Start

1. Run setup.sh

```bash
chmod +x setup.sh
./setup.sh
```

2. Install required system dependencies:

```bash
# Install SQLite vector extension
# Download from https://github.com/asg017/sqlite-vec
```

3. Run example

```bash
cargo run --example Research_Questionaire --features=sqlx
```

### Basic Usage

```rust
use pyano::{
    agent::agent_builder::AgentBuilder,
    chain::sequential_chain::Chain,
    ModelManager,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the model manager
    let model_manager = Arc::new(ModelManager::new());

    // Create agents for your workflow
    let agent_1 = Arc::new(Mutex::new(
        AgentBuilder::new()
            .with_name("Content Generator")
            .with_system_prompt("You are a content generator.")
            .with_user_prompt("Generate content about AI.")
            .with_stream(true)
            .build()
    ));

    // Create and run a chain
    let mut chain = Chain::new()
        .add_agent(agent_1);

    chain.run().await?;

    Ok(())
}
```

## 🔧 Advanced Features

### Vector Storage

```rust
use pyano::{
    vectorstore::sqlite_vec::StoreBuilder,
    embedding::embedding_models::{EmbeddingModels, TextEmbeddingModels},
};

// Initialize vector store
let store = StoreBuilder::new()
    .embedder(embedder)
    .db_name("my_app")
    .table("documents")
    .build()
    .await?;

// Add documents and search
store.add_documents(&documents, &VecStoreOptions::default()).await?;
let results = store.similarity_search("query", 5, &options).await?;
```

### Model Management

```rust
// Start model manager server
let manager = Arc::new(ModelManager::new());
let server = ModelManagerServer::new(manager);
server.run("127.0.0.1:8090").await?;

// Connect client
let client = ModelManagerClient::new("http://127.0.0.1:8090");
```

## 🛠 Tools

Pyano includes several built-in tools:

-   **Web Scraper**: Extract content from websites
-   **DuckDuckGo Search**: Perform web searches
-   **Command Executor**: Execute system commands
-   **Vector Store**: Store and query vector embeddings

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

-   Built with Rust 🦀
-   Uses SQLite and sqlite-vec for vector storage
-   Inspired by various AI frameworks and tools

## 📚 Documentation

For detailed documentation and examples, check out:

-   `/docs` directory for usage examples
-   `/examples` directory for sample applications
-   Code documentation with `cargo doc --open`

## 🔮 Future Plans

-   [ ] Support for more model types
-   [ ] Support for OpenAI, Anthropic, Together and other Centralised AI providers
-   [ ] Enhanced memory management
-   [ ] Additional vector store backends including Chroma, LanceDB
-   [ ] More built-in tools and agents
-   [ ] Improved documentation and examples

---

Built with ❤️ using Rust

## Star History

<a href="https://star-history.com/#pyano/pyano-framework&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=pyano/pyano-framework&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=pyano/pyano-framework&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=pyano/pyano-framework&type=Date" />
 </picture>
</a>
