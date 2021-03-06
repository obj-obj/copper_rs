use clap::{Parser, Subcommand};
use copper_lib::{
	api::mojang::{get_profile, get_version_manifest},
	instance::Instance,
	Directories,
};
use std::{error::Error, fs::create_dir_all, process::Command};
use tokio::task::JoinHandle;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
	#[clap(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Manage instances
	Instance {
		#[clap(subcommand)]
		command: InstanceCommand,
	},
	/// Updates the cache (contains version manifest and version profiles).
	/// The first run will take a while.
	Update,
}

#[derive(Subcommand)]
enum InstanceCommand {
	/// Testing command
	Test { version: String },
	/// Lists all instances
	List,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	// Initialize logging
	let subscriber = FmtSubscriber::builder().without_time().finish();
	tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize logging!");

	// Main init
	let args = Args::parse();
	let dir = Directories::new("copper_launcher");

	info!("Config directory: {}", dir.config.display());
	info!("Cache directory: {}", dir.cache.display());

	match &args.command {
		Commands::Instance { command } => match command {
			InstanceCommand::Test { version } => {
				let manifest = get_version_manifest(dir.versions.join("manifest.json")).await;
				let instance = Instance::new(
					"test",
					&dir,
					get_profile(
						manifest.versions.get(version).unwrap(),
						dir.versions.join(version).join("profile.json"),
					)
					.await
					.unwrap(),
				)
				.await;
				instance.launch().await;
			}

			InstanceCommand::List => {
				info!("Yes this is just ls");
				Command::new("ls").arg(&dir.instances).spawn()?;
			}
		},

		Commands::Update => {
			info!("Updating profiles...");
			let version_manifest = get_version_manifest(dir.versions.join("manifest.json")).await;

			// TODO Error handling
			let mut handles: Vec<JoinHandle<()>> = Vec::new();
			for version in version_manifest.versions {
				// Want multithreading? Spawn a task for every version of minecraft!
				let path = dir.versions.join(&version.0).join("profile.json");
				create_dir_all(path.parent().unwrap())?;
				handles.push(tokio::spawn(async move {
					get_profile(&version.1, &path).await.unwrap();
				}));
			}
			for handle in handles {
				handle.await?;
			}
			info!("Updated profiles.");
		}
	}

	Ok(())
}
