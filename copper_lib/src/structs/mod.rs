//! Data structs for version profiles.
//!
//! Main structs:
//!
//! [Manifest]: Fetched from <https://launchermeta.mojang.com/mc/game/version_manifest.json>.
//!
//! [Profile]: Fetched from URLs contained in the [Manifest].

pub mod asset_index;
pub use asset_index::AssetIndex;

pub mod version_manifest;
pub use version_manifest::VersionManifest;

pub mod profile;
pub use profile::Profile;
use profile::*;

impl Rule {
	pub fn is_true(&self, demo: bool, custom_resolution: bool) -> bool {
		for rule in &self.rules {
			if !rule.is_true(demo, custom_resolution) {
				return false;
			}
		}
		true
	}
}

impl RuleItem {
	pub fn is_true(&self, demo: bool, custom_resolution: bool) -> bool {
		let features = match &self.features {
			Some(features) => features.is_true(demo, custom_resolution),
			None => true,
		};
		let os = match &self.os {
			Some(os) => os.is_true(),
			None => true,
		};

		match &self.action {
			RuleAction::Allow => features && os,
			RuleAction::Disallow => !(features && os),
		}
	}
}

impl RuleItemFeatures {
	pub fn is_true(&self, demo: bool, custom_resolution: bool) -> bool {
		let demo = match self.is_demo_user {
			Some(is_demo_user) => demo == is_demo_user,
			None => true,
		};
		let custom_resolution = match self.has_custom_resolution {
			Some(has_custom_resolution) => custom_resolution == has_custom_resolution,
			None => true,
		};

		demo && custom_resolution
	}
}

impl RuleItemOs {
	pub fn is_true(&self) -> bool {
		let arch = match &self.arch {
			Some(arch) => match os_info::get().bitness() {
				os_info::Bitness::X32 => arch == "x32",
				os_info::Bitness::X64 => arch == "x86",
				_ => true,
			},
			None => true,
		};
		let name = match &self.name {
			Some(name) => {
				name == match os_info::get().os_type() {
					os_info::Type::Macos => "osx",
					os_info::Type::Windows => "windows",
					_ => "linux",
				}
			}
			None => true,
		};
		// TODO parse version. Not being done right now because it's only valid on windows, and who uses a version of windows less than 10 these days?
		let version = true;

		arch && name && version
	}
}

impl Library {
	pub fn is_active(&self) -> bool {
		let mut active = true;
		if let Some(rules) = &self.rules {
			for rule in rules {
				// The `demo` and `custom_resolution` values only show up on rules applied to java arguments, so they don't matter for library rules.
				if !rule.is_true(false, false) {
					active = false;
					break;
				}
			}
		}
		active
	}
}
