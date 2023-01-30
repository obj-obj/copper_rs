mod helpers;

use crate::{
	api::mojang::get_asset_index,
	store::{fetch, update_file},
	structs::{
		profile::{Argument, Arguments, RuleValue},
		AssetIndex, Profile,
	},
	Paths, Result,
};
use helpers::generate_classpath;
use std::{
	fs::{self, create_dir_all, File},
	io,
	path::PathBuf,
	process::Command,
};
use tokio::task::JoinHandle;
use tracing::info;
use zip::ZipArchive;

pub struct Instance {
	asset_index: AssetIndex,
	classpath: String,
	profile: Profile,

	path: Paths,
	natives: PathBuf,
}

impl Instance {
	pub async fn new(path: &Paths, profile: Profile) -> Result<Self> {
		let natives = path.natives.join(&profile.id);
		create_dir_all(&natives)?;
		Ok(Self {
			asset_index: get_asset_index(&profile.asset_index).await?,
			classpath: generate_classpath(&profile).await?,
			profile,

			path: path.clone(),
			natives,
		})
	}

	pub async fn launch(&self) -> Result<()> {
		let asset_handles = self.update_assets().await?;
		let native_handles = self.update_natives().await?;
		for handle in asset_handles {
			handle.await?;
		}
		for handle in native_handles {
			handle.await?;
		}

		let jvm_args = self.parse_jvm_arguments();
		let game_args = self.parse_game_arguments(false, false);

		info!("Launching {}...", self.profile.id);
		Command::new("java")
			.current_dir(&self.path.config)
			.args(jvm_args)
			.args(game_args)
			.spawn()?
			.wait()?;

		Ok(())
	}

	pub async fn update_assets(&self) -> Result<Vec<JoinHandle<()>>> {
		let mut handles = Vec::new();
		let assets_root = self.path.assets.join("objects");

		update_file(
			None,
			self.profile.asset_index.url.clone(),
			&self.path.asset_indexes.join(format!(
				"{}.json",
				self.profile.asset_index.id.as_ref().unwrap()
			)),
		)
		.await?;
		for asset in &self.asset_index.objects {
			let entry = asset.1;
			let doublehash = format!("{}/{}", &entry.hash[..2], entry.hash);
			let path = assets_root.join(&doublehash);
			fs::create_dir_all(path.parent().unwrap())?;

			handles.push(tokio::spawn(async move {
				update_file(
					None,
					format!("https://resources.download.minecraft.net/{doublehash}"),
					&path,
				)
				.await
				.unwrap();
			}));
		}

		Ok(handles)
	}

	pub async fn update_natives(&self) -> Result<Vec<JoinHandle<()>>> {
		let mut handles = Vec::new();

		for library in &self.profile.libraries {
			if !library.is_active() {
				continue;
			}

			let library = library.clone();
			let natives_path = self.natives.clone();

			handles.push(tokio::spawn(async move {
				if !library.is_active() {
					return;
				}

				if let Some(classifiers) = &library.downloads.classifiers {
					// TODO make this less scuffed
					let natives = match os_info::get().os_type() {
						os_info::Type::Windows => match &classifiers.natives_windows {
							Some(classifiers) => classifiers,
							None => return,
						},
						os_info::Type::Macos => match &classifiers.natives_macos {
							Some(classifiers) => classifiers,
							None => return,
						},
						_ => match &classifiers.natives_linux {
							Some(classifiers) => classifiers,
							None => return,
						},
					};

					let path = fetch(None, natives.url.clone()).await.unwrap().path;
					let mut zip = ZipArchive::new(File::open(path).unwrap()).unwrap();
					for i in 0..zip.len() {
						let mut file = zip.by_index(i).unwrap();
						let path = PathBuf::from(file.name());

						let extension = path.extension();
						if extension.is_none() {
							continue;
						}
						let extension = extension.unwrap();

						if extension == "so" || extension == "dll" || extension == "dylib" {
							let destination_path = natives_path.join(file.name());
							if !destination_path.exists() {
								let mut destination = File::create(destination_path).unwrap();

								io::copy(&mut file, &mut destination).unwrap();
								info!("Extracted native {}", file.name());
							}
						}
					}
				}
			}));
		}

		Ok(handles)
	}

	// Minecraft/JVM arguments
	pub fn parse_game_arguments(&self, demo: bool, custom_resolution: bool) -> Vec<String> {
		let mut args = Vec::new();

		match &self.profile.arguments {
			Arguments::NewArguments(arguments) => {
				self.parse_arguments_vec(&mut args, &arguments.game, demo, custom_resolution);
			}
			Arguments::OldArguments(arguments) => {
				let mut new_arguments = Vec::new();
				for argument in arguments.split(' ') {
					new_arguments.push(Argument::String(argument.into()));
				}
				self.parse_arguments_vec(&mut args, &new_arguments, demo, custom_resolution);
			}
		}

		args
	}

	pub fn parse_jvm_arguments(&self) -> Vec<String> {
		let mut args = Vec::new();

		match &self.profile.arguments {
			Arguments::NewArguments(arguments) => {
				self.parse_arguments_vec(&mut args, &arguments.jvm, false, false);
				args.push(self.profile.main_class.clone());
			}
			Arguments::OldArguments(_a) => {
				let new_arguments = vec![Argument::String(
					"-Djava.library.path=${natives_directory}".into(),
				)];
				self.parse_arguments_vec(&mut args, &new_arguments, false, false);
				args.push(self.profile.main_class.clone());
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

	pub fn parse_argument(&self, args: &mut Vec<String>, arg: &str) {
		let mut arg = arg.to_string();
		arg = arg.replace("${assets_index_name}", &self.profile.assets);
		arg = arg.replace("${assets_root}", self.path.assets.to_str().unwrap());
		arg = arg.replace("${classpath}", &self.classpath);
		arg = arg.replace("${game_directory}", self.path.config.to_str().unwrap());
		arg = arg.replace("${launcher_name}", "Copper Launcher");
		arg = arg.replace("${launcher_version}", "v0.1.0");
		arg = arg.replace("${natives_directory}", self.natives.to_str().unwrap());
		arg = arg.replace("${version_name}", &self.profile.id);
		arg = arg.replace("${version_type}", &self.profile.version_type);

		args.push(arg);
	}
}
