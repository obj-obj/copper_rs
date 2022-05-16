use crate::Mod;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
	pub id: String,
	pub name: String,
	pub mods: Vec<Mod>,
}
impl Config {
	pub fn new(id: impl ToString, name: impl ToString) -> Self {
		Self {
			id: id.to_string(),
			name: name.to_string(),
			mods: Vec::new(),
		}
	}
}
