use super::Config;
use crate::{
	api::mojang::get_asset_index,
	data::{
		profile::{Argument, Arguments, RuleValue},
		AssetIndex, Profile,
	},
	download_if_invalid, Directories,
};
use std::{
	fs::{self, create_dir_all, File},
	io::{self, Write},
	path::{Path, PathBuf},
	process::Command,
};
use tracing::{error, info, warn};
use zip::ZipArchive;

pub struct Instance {
	classpath: String,
	config: Config,
	dir: Directories,

	asset_index: AssetIndex,
	profile: Profile,

	path: PathBuf,
	assets_dir: PathBuf,
	jar_path: PathBuf,
}
impl Instance {
	pub async fn new(name: impl ToString, dir: &Directories, profile: Profile) -> Self {
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

		Self {
			classpath: generate_classpath(&profile, dir, &jar_path),
			config,
			dir: dir.to_owned(),

			asset_index: get_asset_index(&profile.asset_index, dir).await.unwrap(),
			profile,

			path: path.to_owned(),
			assets_dir: dir.cache.join("assets"),
			jar_path,
		}
	}

	pub async fn launch(&self) {
		self.update_assets().await;
		self.update_client().await;
		self.update_libraries().await;

		let args = self.parse_arguments(false, false);
		let mut command = Command::new("java");
		command.current_dir(&self.path).args(args);

		info!("Launching {}...", self.profile.id);
		command
			.spawn()
			.expect("Failed to launch Minecraft instance!")
			.wait_with_output()
			.unwrap();
	}

	// Minecraft/JVM arguments
	pub fn parse_arguments(&self, demo: bool, custom_resolution: bool) -> Vec<String> {
		let mut args = Vec::new();

		match &self.profile.arguments {
			Arguments::NewArguments(arguments) => {
				self.parse_arguments_vec(&mut args, &arguments.jvm, demo, custom_resolution);
				self.add_other_jvm_arguments(&mut args);
				self.parse_arguments_vec(&mut args, &arguments.game, demo, custom_resolution)
			}
			Arguments::OldArguments(arguments) => {
				self.add_other_jvm_arguments(&mut args);
				let mut new_arguments = Vec::new();
				for argument in arguments.split(" ") {
					new_arguments.push(Argument::String(argument.into()));
				}
				self.parse_arguments_vec(&mut args, &new_arguments, demo, custom_resolution)
			}
		}

		args
	}

	pub fn parse_arguments_vec(
		&self,
		args: &mut Vec<String>,
		arguments: &Vec<Argument>,
		demo: bool,
		custom_resolution: bool,
	) {
		for argument in arguments {
			match argument {
				Argument::String(value) => self.parse_argument(args, value),
				Argument::Rule(rule) => {
					if rule.is_true(demo, custom_resolution) {
						match &rule.value {
							RuleValue::String(value) => self.parse_argument(args, value),
							RuleValue::Vec(values) => {
								for value in values {
									self.parse_argument(args, value);
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

	pub fn parse_argument(&self, args: &mut Vec<String>, arg: impl ToString) {
		let mut arg = arg.to_string();
		arg = arg.replace("${assets_index_name}", &self.profile.assets);
		arg = arg.replace("${assets_root}", self.assets_dir.to_str().unwrap());
		arg = arg.replace("${classpath}", &self.classpath);
		arg = arg.replace("${game_directory}", self.path.to_str().unwrap());
		arg = arg.replace("${launcher_name}", "Copper Launcher");
		arg = arg.replace("${launcher_version}", "v0.1.0");
		arg = arg.replace("${natives_directory}", self.dir.natives.to_str().unwrap());
		arg = arg.replace("${version_name}", &self.profile.id);
		arg = arg.replace("${version_type}", &self.profile.version_type);

		args.push(arg);
	}

	// Assets
	pub async fn update_assets(&self) {
		info!("Updating assets for {}", self.profile.id);

		let mut handles = Vec::new();
		let assetspath = self.dir.assets.join("objects");
		for asset in &self.asset_index.objects {
			let entry = asset.1.to_owned();
			let doublehash = format!("{}/{}", &entry.hash[..2], entry.hash);
			let hashpath = assetspath.join(&doublehash);
			create_dir_all(hashpath.parent().unwrap()).unwrap();
			handles.push(tokio::spawn(async move {
				download_if_invalid(
					&hashpath,
					format!("https://resources.download.minecraft.net/{doublehash}"),
					&entry.hash,
				)
				.await
				.unwrap();
			}));
		}
		for handle in handles {
			handle.await.unwrap();
		}
		info!("Updated assets for {}.", self.profile.id);
	}

	// Libraries
	pub async fn update_libraries(&self) {
		info!("Updating libraries for {}...", self.profile.id);

		let mut handles = Vec::new();
		for library in &self.profile.libraries {
			let dir = self.dir.clone();
			let library = library.to_owned();
			handles.push(tokio::spawn(async move {
				if !library.is_active() {
					return;
				}

				if let Some(download) = &library.downloads.artifact {
					if let Some(path) = &download.path {
						let path = dir.libraries.join(path);
						create_dir_all(path.parent().unwrap()).unwrap();
						download_if_invalid(&path, &download.url, &download.sha1)
							.await
							.unwrap();
					}
				}

				if let Some(classifiers) = &library.downloads.classifiers {
					// TODO make this less scuffed
					let natives = match os_info::get().os_type() {
						os_info::Type::Macos => match &classifiers.natives_macos {
							Some(classifiers) => classifiers,
							None => return,
						},
						os_info::Type::Windows => match &classifiers.natives_windows {
							Some(classifiers) => classifiers,
							None => return,
						},
						_ => match &classifiers.natives_linux {
							Some(classifiers) => classifiers,
							None => return,
						},
					};

					if let Some(path) = &natives.path {
						let path = dir.libraries.join(path);
						create_dir_all(path.parent().unwrap()).unwrap();

						download_if_invalid(&path, &natives.url, &natives.sha1)
							.await
							.unwrap();

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
								let destination_path = dir.natives.join(file.name());
								if !destination_path.exists() {
									let mut destination = File::create(destination_path).unwrap();

									info!("Extracting native {}...", file.name());
									io::copy(&mut file, &mut destination).unwrap();
								}
							}
						}
					}
				}
			}));
		}

		for handle in handles {
			handle.await.unwrap();
		}

		info!("Updated libraries for {}.", self.profile.id);
	}

	pub async fn update_client(&self) {
		download_if_invalid(
			&self.jar_path,
			&self.profile.downloads.client.url,
			&self.profile.downloads.client.sha1,
		)
		.await
		.unwrap();
	}
}

pub fn generate_classpath(
	profile: &Profile,
	dir: &Directories,
	jar_path: impl AsRef<Path>,
) -> String {
	let mut classpath = String::new();
	for library in &profile.libraries {
		match &library.downloads.artifact {
			Some(artifact) => match &artifact.path {
				Some(path) => {
					if !library.is_active() {
						continue;
					}

					classpath += &format!("{}:", dir.libraries.join(path).display())
				}
				None => {}
			},
			None => {}
		}
	}
	classpath += jar_path.as_ref().to_str().unwrap();
	classpath
}

fn write_default_config(path: impl AsRef<Path>, id: impl ToString, name: impl ToString) -> Config {
	let config = Config::new(id, name);
	let mut file = File::create(path).unwrap();
	write!(file, "{}", toml::ser::to_string_pretty(&config).unwrap()).unwrap();
	config
}
