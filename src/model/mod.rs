pub mod manager;

pub mod error;

mod types;
pub mod process;
pub mod config_loader;
pub mod system_memory;
pub mod manager_trait;
pub mod adapters;
pub mod utils;
pub mod state;

mod client;
mod server;

pub use types::*;
pub use manager::ModelManager;
pub use client::ModelManagerClient;
pub use server::ModelManagerServer;
pub use system_memory::SystemMemory;
pub use manager_trait::ModelManagerInterface;
