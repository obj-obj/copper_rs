//! A library to do launcher stuff, such as downloading versions/libraries from mojang and interacting with curseforge/modrinth APIs and installing modloaders.
//!
//! Might be split up into multiple crates in the future.

pub mod api;
mod directories;
pub use directories::*;
pub mod instance;
mod modpack;
pub use modpack::*;
pub mod version;
