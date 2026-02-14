// MCM-Finder Library
// Multi-provider Minecraft mod search tool

pub mod models;
pub mod providers;
pub mod services;
pub mod error;
pub mod db;

// Re-export commonly used types
pub use error::{Error, Result};
