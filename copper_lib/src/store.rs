//! Assumptions: `{STORE_BASE}` already exists

use crate::Result;
use data_encoding::HEXLOWER;
use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha512};
use std::{
	collections::HashMap,
	error::Error,
	fmt,
	fs::{self, File},
	io::Write,
	os::unix,
	path::PathBuf,
};
use std::{io, sync::Mutex};
use tracing::warn;

lazy_static! {
	static ref STORE_BASE: PathBuf = dirs::cache_dir().unwrap().join("copper_launcher/store");
	static ref URL_CACHE: Mutex<HashMap<String, String>> = {
		if let Ok(file) = File::open(STORE_BASE.join("cache.json")) {
			if let Ok(data) = serde_json::from_reader(file) {
				return Mutex::new(data);
			}
		}

		Mutex::new(HashMap::new())
	};
}

pub fn save_url_cache() -> Result<()> {
	let file = File::create(STORE_BASE.join("cache.json"))?;
	serde_json::to_writer(file, &*URL_CACHE)?;
	Ok(())
}

pub enum StoreInput {
	Sha512(String),
	URL(String),
}

#[derive(Debug)]
pub struct StoreOutput {
	pub path: PathBuf,
	pub sha512: String,
}

/// Save data to the store from a reader.
pub fn save(data: impl AsRef<[u8]>, input: StoreInput) -> Result<StoreOutput> {
	let sha512 = match input {
		StoreInput::Sha512(hash) => hash,
		StoreInput::URL(url) => {
			let mut hasher = Sha512::new();
			hasher.update(&data);
			let hash = HEXLOWER.encode(&hasher.finalize());
			URL_CACHE.lock().unwrap().insert(url, hash.clone());
			hash
		}
	};

	let path = STORE_BASE.join(&sha512);
	let mut file = File::create(&path)?;
	file.write_all(data.as_ref())?;

	Ok(StoreOutput { path, sha512 })
}

#[derive(Debug)]
pub enum GetError {
	CorruptedFile,
	URLNotInCache,
}
impl fmt::Display for GetError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::CorruptedFile => write!(f, "File in store is corrupted"),
			Self::URLNotInCache => write!(f, "Requested URL not in cache"),
		}
	}
}
impl Error for GetError {}

/// Get data from the store as a reader.
pub fn get(input: StoreInput) -> Result<StoreOutput> {
	let sha512 = match input {
		StoreInput::Sha512(hash) => hash,
		StoreInput::URL(url) => {
			if let Some(hash) = URL_CACHE.lock().unwrap().get(&url) {
				hash.to_owned()
			} else {
				return Err(Box::new(GetError::URLNotInCache));
			}
		}
	};

	let path = STORE_BASE.join(&sha512);
	let mut file = File::open(&path)?;
	// TODO: is checking the hashes of files really needed?
	if cfg!(not(debug_assertions)) {
		let mut hasher = Sha512::new();
		io::copy(&mut file, &mut hasher)?;
		if HEXLOWER.encode(&hasher.finalize()) != sha512 {
			return Err(Box::new(GetError::CorruptedFile));
		}
	}
	Ok(StoreOutput { path, sha512 })
}

/// Get data from the store, or download it as a fallback.
pub async fn fetch(sha512: Option<String>, url: String) -> Result<StoreOutput> {
	if let Some(sha512) = &sha512 {
		let store = get(StoreInput::Sha512(sha512.to_string()));
		if store.is_err() {
			warn!("{}", store.unwrap_err());
		} else {
			// info!("{sha512} in cache...");
			return store;
		}
	}
	let store = get(StoreInput::URL(url.clone()));
	if store.is_err() {
		warn!("{}", store.unwrap_err());
	} else {
		// info!("{} in cache...", store.as_ref().unwrap().sha512);
		return store;
	}

	// info!("Downloading {url}...");
	let data = reqwest::get(&url).await?.bytes().await?;
	let input = match sha512 {
		Some(hash) => StoreInput::Sha512(hash),
		None => StoreInput::URL(url),
	};
	save(&data, input)
}

/// Save data from a URL to the store, always updating the sha512 cache.
pub async fn force_update(url: String) -> Result<StoreOutput> {
	let data = reqwest::get(&url).await?.bytes().await?;
	save(&data, StoreInput::URL(url))
}

/// Get data from the store and deserialize it to a struct.
pub async fn fetch_data<T>(sha512: Option<String>, url: String) -> Result<T>
where
	T: DeserializeOwned,
{
	match serde_json::from_reader(File::open(fetch(sha512, url).await?.path)?) {
		Ok(data) => Ok(data),
		Err(err) => Err(Box::new(err)),
	}
}

/// Fetch a file, and symlink it to a directory.
pub async fn update_file(
	sha512: Option<String>,
	url: String,
	path: &PathBuf,
) -> Result<StoreOutput> {
	let store = fetch(sha512, url).await?;
	if path.exists() {
		return Ok(store);
	}

	fs::create_dir_all(path.parent().unwrap())?;
	unix::fs::symlink(&store.path, path)?;
	Ok(store)
}
