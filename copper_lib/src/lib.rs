//! A library to do launcher stuff, such as downloading versions/libraries from mojang and interacting with curseforge/modrinth APIs and installing modloaders.
//!
//! Might be split up into multiple crates in the future.

pub mod api;
mod directories;
pub use directories::*;
pub mod instance;
pub mod store;
pub mod structs;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
