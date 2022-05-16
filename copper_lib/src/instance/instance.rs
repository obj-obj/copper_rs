use super::Config;
use crate::{version::Profile, Directories};
use sha1::{Digest, Sha1};
use std::{
	error::Error,
	fs::{self, create_dir_all, File},
	io::{self, Write},
	path::PathBuf,
};
use tracing::{error, info, warn};
use zip::ZipArchive;

pub struct Instance {
	classpath: Option<String>,
	config: Config,
	dir: Directories,
	path: PathBuf,
	profile: Profile,
	version_dir: PathBuf,
}
impl Instance {
	pub fn new(name: impl ToString, dir: &Directories, profile: Profile) -> Self {
		let path = dir.instances.join(name.to_string());
		create_dir_all(&path).unwrap();

		let config_path = path.join("instance.toml");
		let config = match fs::read_to_string(&config_path) {
			Ok(data) => match toml::from_str::<Config>(&data) {
				Ok(config) => config,
				Err(err) => {
					error!("Error deserializing config file!\n\t{err}\n\tCreating a new file with default config...");
					write_default_config(&config_path, &profile.id, name)
				}
			},
			Err(err) => {
				warn!("Error reading from config file!\n\t{err}\n\tCreating new file with default config...");
				write_default_config(&config_path, &profile.id, name)
			}
		};

		Self {
			classpath: None,
			config,
			dir: dir.to_owned(),
			path: path.to_owned(),
			version_dir: dir.versions.join(&profile.id),
			profile,
		}
	}

	// Classpath
	pub fn classpath(&mut self) -> String {
		match &self.classpath {
			Some(classpath) => classpath.to_string(),
			None => {
				let classpath = self.generate_classpath();
				self.classpath = Some(classpath.clone());
				classpath
			}
		}
	}

	pub fn generate_classpath(&self) -> String {
		let mut classpath = String::new();
		for library in &self.profile.libraries {
			match &library.downloads.artifact {
				Some(artifact) => match &artifact.path {
					Some(path) => classpath += &format!("{path};"),
					None => {}
				},
				None => {}
			}
		}
		classpath += &format!("{};", self.version_dir.join("client.jar").display());
		classpath += &self.profile.main_class;
		classpath
	}

	// Libraries
	pub async fn update_libraries(&self) -> Result<(), Box<dyn Error>> {
		info!("Updating libraries for {}...", self.profile.id);
		for library in &self.profile.libraries {
			if let Some(rules) = &library.rules {
				let mut skip = false;
				for rule in rules {
					// The `demo` and `custom_resolution` values only show up on rules applied to java arguments, so they don't matter for library rules.
					if !rule.is_true(false, false) {
						skip = true;
						break;
					}
				}
				if skip {
					continue;
				}

				if let Some(download) = &library.downloads.artifact {
					if let Some(path) = &download.path {
						let path = self.dir.libraries.join(path);
						create_dir_all(path.parent().unwrap())?;
						download_if_invalid(&path, &download.url, &download.sha1).await?;
					}
				}

				if let Some(classifiers) = &library.downloads.classifiers {
					// TODO make this less scuffed
					let natives = match os_info::get().os_type() {
						os_info::Type::Macos => match &classifiers.natives_macos {
							Some(classifiers) => classifiers,
							None => continue,
						},
						os_info::Type::Windows => match &classifiers.natives_windows {
							Some(classifiers) => classifiers,
							None => continue,
						},
						_ => match &classifiers.natives_linux {
							Some(classifiers) => classifiers,
							None => continue,
						},
					};

					if let Some(path) = &natives.path {
						let path = self.dir.libraries.join(path);
						create_dir_all(path.parent().unwrap())?;
						let natives_dir = &self.version_dir.join("natives");
						create_dir_all(natives_dir)?;

						download_if_invalid(&path, &natives.url, &natives.sha1).await?;

						let mut zip = ZipArchive::new(File::open(path)?)?;
						for i in 0..zip.len() {
							let mut file = zip.by_index(i)?;
							let path = PathBuf::from(file.name());

							let extension = path.extension();
							if let None = extension {
								continue;
							}
							let extension = extension.unwrap();

							if extension == "so" || extension == "dll" || extension == "dylib" {
								let destination_path = natives_dir.join(file.name());
								if !destination_path.exists() {
									let mut destination = File::create(destination_path)?;

									info!("Extracting native {}...", file.name());
									io::copy(&mut file, &mut destination)?;
								}
							}
						}
					}
				}
			}
		}

		info!("Updated libraries for {}.", self.profile.id);
		Ok(())
	}

	pub async fn update_client(&self) -> Result<(), Box<dyn Error>> {
		download_if_invalid(
			&self.version_dir.join("client.jar"),
			&self.profile.downloads.client.url,
			&self.profile.downloads.client.sha1,
		)
		.await?;

		Ok(())
	}
}

async fn download_if_invalid(
	path: &PathBuf,
	url: impl ToString,
	sha1: impl ToString,
) -> Result<(), Box<dyn Error>> {
	let mut file = File::options()
		.read(true)
		.write(true)
		.create(true)
		.open(path)?;

	let mut hasher = Sha1::new();
	io::copy(&mut file, &mut hasher)?;
	let hash = base16ct::lower::encode_string(&hasher.finalize());
	if hash != sha1.to_string() {
		info!(
			"{:?} is out of date, updating...",
			path.file_name().unwrap()
		);

		file = File::create(&path)?;
		let data = reqwest::get(url.to_string()).await?.bytes().await?;
		file.write_all(&data)?;
		info!("Updated {:?}.", path.file_name().unwrap());
	}

	Ok(())
}

fn write_default_config(path: &PathBuf, id: impl ToString, name: impl ToString) -> Config {
	let config = Config::new(id, name);
	let mut file = File::create(path).unwrap();
	write!(file, "{}", toml::ser::to_string_pretty(&config).unwrap()).unwrap();
	config
}
