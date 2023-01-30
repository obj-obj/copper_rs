use clap::{Parser, Subcommand};
use copper_lib::{
	api::mojang::{fetch_version_manifest, get_profile},
	instance::Instance,
	store::save_url_cache,
	Paths,
};
use std::error::Error;
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
	/// Launch specific version of Minecraft
	Launch { version: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
	// Initialize logging
	tracing::subscriber::set_global_default(FmtSubscriber::default())
		.expect("Failed to initialize logging!");

	// Main init
	let args = Args::parse();
	let path = Paths::new("copper_launcher".to_string())?;

	info!("Config directory: {}", path.config.display());
	info!("Cache directory: {}", path.cache.display());

	match &args.command {
		Commands::Launch { version } => {
			let manifest = fetch_version_manifest().await?;
			let instance = Instance::new(
				&path,
				get_profile(manifest.versions.get(version).unwrap())
					.await
					.unwrap(),
			)
			.await?;
			instance.launch().await?;
			save_url_cache()?;
		}
	}

	Ok(())
}
