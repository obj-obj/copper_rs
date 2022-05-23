//! A library to do launcher stuff, such as downloading versions/libraries from mojang and interacting with curseforge/modrinth APIs and installing modloaders.
//!
//! Might be split up into multiple crates in the future.

use data::profile::Download;
use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use sha1::{Digest, Sha1};
use std::{
	error::Error,
	fs::{self, File},
	io,
	path::PathBuf,
};
use tracing::info;

pub mod api;
mod directories;
pub use directories::*;
pub mod instance;
mod modpack;
pub use modpack::*;
pub mod data;

pub(crate) async fn get_from_download<T>(
	download: &Download,
	path: &PathBuf,
) -> Result<T, Box<dyn Error>>
where
	T: DeserializeOwned,
{
	get_from_url(&path, &download.url, &download.sha1).await
}

pub(crate) async fn get_from_url<T>(
	path: &PathBuf,
	url: impl IntoUrl,
	sha1: impl ToString,
) -> Result<T, Box<dyn Error>>
where
	T: DeserializeOwned,
{
	Ok(serde_json::de::from_slice(
		&download_if_invalid(path, url, sha1).await?,
	)?)
}

pub(crate) async fn download_if_invalid(
	path: &PathBuf,
	url: impl IntoUrl,
	sha1: impl ToString,
) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut file = File::options()
		.read(true)
		.write(true)
		.create(true)
		.open(path)?;

	let mut hasher = Sha1::new();
	io::copy(&mut file, &mut hasher)?;
	let hash = base16ct::lower::encode_string(&hasher.finalize());
	let name = path.file_name().unwrap().to_str().unwrap();
	if hash != sha1.to_string() {
		info!("{name} is invalid/missing, updating...");

		let data = reqwest::get(url).await?.bytes().await?;
		fs::write(path, &data)?;
		info!("Updated {name}.");
		return Ok(data.to_vec());
	}

	info!("{name} is valid.");
	return Ok(fs::read(path)?);
}
