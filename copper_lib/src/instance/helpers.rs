use crate::{store::fetch, structs::Profile, Result};

pub async fn generate_classpath(profile: &Profile) -> Result<String> {
	let mut classpath = Vec::new();

	for library in &profile.libraries {
		if !library.is_active() {
			continue;
		}

		if let Some(artifact) = &library.downloads.artifact {
			classpath.push(format!(
				"{}",
				fetch(None, artifact.url.clone()).await?.path.display()
			));
		}
	}
	classpath.push(format!(
		"{}",
		fetch(None, profile.downloads.client.url.clone())
			.await?
			.path
			.display()
	));

	Ok(classpath.join(":"))
}
