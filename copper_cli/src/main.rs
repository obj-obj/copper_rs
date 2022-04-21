use clap::{Parser, Subcommand};
use copper_lib::api::mojang::{get_version, get_version_manifest};
use std::{fs::create_dir_all, process::Command};
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
	/// Creates a new instance
	Create { name: String },
	/// Lists all instances
	List,
}

#[tokio::main]
async fn main() {
	// Directory definitions
	let folder_name = "copper_launcher";
	let cache_dir = dirs::cache_dir().unwrap().join(folder_name);
	let config_dir = dirs::config_dir().unwrap().join(folder_name);

	let instance_dir = config_dir.join("instances");
	let versions_dir = cache_dir.join("versions");

	// Create directories if they don't already exist
	create_dir_all(&cache_dir).unwrap();
	create_dir_all(&config_dir).unwrap();
	create_dir_all(&instance_dir).unwrap();
	create_dir_all(&versions_dir).unwrap();

	// Initialize logging
	let subscriber = FmtSubscriber::builder().without_time().finish();
	tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize logging!");

	// Main init
	let args = Args::parse();

	match &args.command {
		Commands::Instance { command } => match command {
			InstanceCommand::Create { name } => {
				create_dir_all(instance_dir.join(name)).unwrap();
			}
			InstanceCommand::List => {
				info!("Yes this is just ls");
				Command::new("ls").arg(&instance_dir).spawn().unwrap();
			}
		},
		Commands::Update => {
			info!("Updating version manifest...");
			let version_manifest =
				get_version_manifest(versions_dir.join("manifest.json").display()).await;
			info!("Done updating version manifest.");
			// TODO Error handling
			for version in version_manifest.versions {
				get_version(
					&version,
					versions_dir.join(format!("{}.json", version.id)).display(),
				)
				.await
				.unwrap();
			}
		}
	}
}
