use super::Config;
use crate::{
	version::{Argument, Arguments, Profile, RuleValue},
	Directories,
};
use sha1::{Digest, Sha1};
use std::{
	fs::{self, create_dir_all, File},
	io::{self, Write},
	path::PathBuf,
	process::Command,
};
use tracing::{error, info, warn};
use zip::ZipArchive;

pub struct Instance {
	classpath: Option<String>,
	config: Config,
	dir: Directories,
	profile: Profile,

	path: PathBuf,
	assets_dir: PathBuf,
	jar_path: PathBuf,
	natives_dir: PathBuf,
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

		let version_dir = dir.versions.join(&profile.id);
		let jar_path = version_dir.join("client.jar");
		let natives_dir = version_dir.join("natives");

		Self {
			classpath: None,
			config,
			dir: dir.to_owned(),
			profile,

			path: path.to_owned(),
			assets_dir: dir.cache.join("assets"),
			jar_path,
			natives_dir,
			version_dir,
		}
	}

	pub async fn launch(&mut self) {
		let classpath = self.classpath();
		self.update_client().await;
		self.update_libraries().await;

		let args = self.parse_arguments(classpath, false, false);
		let mut command = Command::new("java")
			.args(args)
			.spawn()
			.expect("Failed to launch Minecraft instance!");
		command.wait_with_output().unwrap();
	}

	// Minecraft/JVM arguments
	pub fn parse_arguments(
		&self,
		classpath: String,
		demo: bool,
		custom_resolution: bool,
	) -> Vec<String> {
		let mut args = Vec::new();

		match &self.profile.arguments {
			Arguments::NewArguments(arguments) => {
				self.parse_new_arguments_vec(
					&mut args,
					&arguments.jvm,
					&classpath,
					demo,
					custom_resolution,
				);
				self.add_other_jvm_arguments(&mut args);
				self.parse_new_arguments_vec(
					&mut args,
					&arguments.game,
					&classpath,
					demo,
					custom_resolution,
				)
			}
			Arguments::OldArguments(arguments) => todo!(),
		}

		args
	}

	pub fn parse_new_arguments_vec(
		&self,
		args: &mut Vec<String>,
		arguments: &Vec<Argument>,
		classpath: &str,
		demo: bool,
		custom_resolution: bool,
	) {
		for argument in arguments {
			match argument {
				Argument::String(value) => self.parse_argument(args, value, classpath),
				Argument::Rule(rule) => {
					if rule.is_true(demo, custom_resolution) {
						match &rule.value {
							RuleValue::String(value) => self.parse_argument(args, value, classpath),
							RuleValue::Vec(values) => {
								for value in values {
									self.parse_argument(args, value, &classpath);
								}
							}
						}
					}
				}
			}
		}
	}

	pub fn add_other_jvm_arguments(&self, args: &mut Vec<String>) {
		args.push(self.profile.main_class.to_string());
	}

	pub fn parse_argument(&self, args: &mut Vec<String>, arg: impl ToString, classpath: &str) {
		let mut arg = arg.to_string();
		arg = arg.replace("${assets_index_name}", &self.profile.assets);
		arg = arg.replace("${assets_root}", self.assets_dir.to_str().unwrap());
		arg = arg.replace("${classpath}", classpath);
		arg = arg.replace("${game_directory}", self.path.to_str().unwrap());
		arg = arg.replace("${launcher_name}", "Copper Launcher");
		arg = arg.replace("${launcher_version}", "v0.1.0");
		arg = arg.replace("${natives_directory}", self.natives_dir.to_str().unwrap());
		arg = arg.replace("${version_name}", &self.profile.id);
		arg = arg.replace("${version_type}", &self.profile.version_type);

		args.push(arg);
	}

	// Classpath
	pub fn classpath(&mut self) -> String {
		if let Some(classpath) = &self.classpath {
			classpath.to_string()
		} else {
			let classpath = self.generate_classpath();
			self.classpath = Some(classpath.clone());
			classpath
		}
	}

	pub fn generate_classpath(&self) -> String {
		let mut classpath = String::new();
		for library in &self.profile.libraries {
			match &library.downloads.artifact {
				Some(artifact) => match &artifact.path {
					Some(path) => {
						if !library.is_active() {
							continue;
						}

						classpath += &format!("{}:", self.dir.libraries.join(path).display())
					}
					None => {}
				},
				None => {}
			}
		}
		classpath += self.jar_path.to_str().unwrap();
		classpath
	}

	// Libraries
	pub async fn update_libraries(&self) {
		info!("Updating libraries for {}...", self.profile.id);

		for library in &self.profile.libraries {
			if !library.is_active() {
				continue;
			}

			if let Some(download) = &library.downloads.artifact {
				if let Some(path) = &download.path {
					let path = self.dir.libraries.join(path);
					create_dir_all(path.parent().unwrap()).unwrap();
					download_if_invalid(&path, &download.url, &download.sha1).await;
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
					create_dir_all(path.parent().unwrap()).unwrap();

					download_if_invalid(&path, &natives.url, &natives.sha1).await;

					let mut zip = ZipArchive::new(File::open(path).unwrap()).unwrap();
					for i in 0..zip.len() {
						let mut file = zip.by_index(i).unwrap();
						let path = PathBuf::from(file.name());

						let extension = path.extension();
						if let None = extension {
							continue;
						}
						let extension = extension.unwrap();

						if extension == "so" || extension == "dll" || extension == "dylib" {
							let destination_path = self.natives_dir.join(file.name());
							if !destination_path.exists() {
								let mut destination = File::create(destination_path).unwrap();

								info!("Extracting native {}...", file.name());
								io::copy(&mut file, &mut destination).unwrap();
							}
						}
					}
				}
			}
		}

		info!("Updated libraries for {}.", self.profile.id);
	}

	pub async fn update_client(&self) {
		download_if_invalid(
			&self.jar_path,
			&self.profile.downloads.client.url,
			&self.profile.downloads.client.sha1,
		)
		.await;
	}
}

async fn download_if_invalid(path: &PathBuf, url: impl ToString, sha1: impl ToString) {
	let mut file = File::options()
		.read(true)
		.write(true)
		.create(true)
		.open(path)
		.unwrap();

	let mut hasher = Sha1::new();
	io::copy(&mut file, &mut hasher).unwrap();
	let hash = base16ct::lower::encode_string(&hasher.finalize());
	if hash != sha1.to_string() {
		info!(
			"{:?} is out of date, updating...",
			path.file_name().unwrap()
		);

		file = File::create(&path).unwrap();
		let data = reqwest::get(url.to_string())
			.await
			.unwrap()
			.bytes()
			.await
			.unwrap();
		file.write_all(&data).unwrap();
		info!("Updated {:?}.", path.file_name().unwrap());
	}
}

fn write_default_config(path: &PathBuf, id: impl ToString, name: impl ToString) -> Config {
	let config = Config::new(id, name);
	let mut file = File::create(path).unwrap();
	write!(file, "{}", toml::ser::to_string_pretty(&config).unwrap()).unwrap();
	config
}
