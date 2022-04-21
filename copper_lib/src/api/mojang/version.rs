use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Version {
	#[serde(alias = "minecraftArguments")]
	pub arguments: Arguments,
	#[serde(alias = "assetIndex")]
	pub asset_index: AssetIndex,
	pub assets: String,
	#[serde(alias = "complianceLevel", default)]
	pub compliance_level: i32,
	#[serde(alias = "javaVersion", default)]
	pub java_version: JavaVersion,
	pub libraries: Vec<Library>,
	pub logging: Option<Logging>,
	#[serde(alias = "mainClass")]
	pub main_class: String,
	#[serde(alias = "minimumLauncherVersion")]
	pub minimum_launcher_version: i32,
	#[serde(alias = "releaseTime")]
	pub release_time: String,
	pub time: String,
	#[serde(alias = "type")]
	pub version_type: String,
}

/* Types that appear in many places */

// Rule
#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
	pub rules: Vec<RuleItem>,
	pub value: RuleValue,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuleItem {
	pub action: String,
	pub features: Option<RuleItemFeatures>,
	pub os: Option<RuleItemOs>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuleItemFeatures {
	pub is_demo_user: Option<bool>,
	pub has_custom_resolution: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuleItemOs {
	pub arch: Option<String>,
	pub name: Option<String>,
	pub version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RuleValue {
	String(String),
	Vec(Vec<String>),
}

// Download
#[derive(Debug, Deserialize, Serialize)]
pub struct Download {
	pub id: Option<String>,
	pub path: Option<String>,
	pub sha1: String,
	pub size: i32,
	pub url: String,
}

/* Main JSON structure */

// arguments
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Arguments {
	NewArguments(NewArguments),
	OldArguments(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewArguments {
	pub game: Vec<Argument>,
	pub jvm: Vec<Argument>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Argument {
	String(String),
	Rule(Rule),
}

// asset_index
#[derive(Debug, Deserialize, Serialize)]
pub struct AssetIndex {
	pub id: String,
	pub sha1: String,
	pub size: i32,
	#[serde(alias = "totalSize")]
	pub total_size: i32,
	pub url: String,
}

// downloads
#[derive(Debug, Deserialize, Serialize)]
pub struct Downloads {
	pub client: Download,
	pub client_mappings: Option<Download>,
	pub server: Option<Download>,
	pub server_mappings: Option<Download>,
}

// java_version
#[derive(Debug, Deserialize, Serialize)]
pub struct JavaVersion {
	pub component: String,
	#[serde(alias = "majorVersion")]
	pub major_version: i32,
}
impl Default for JavaVersion {
	fn default() -> Self {
		Self {
			component: String::from("java-runtime-beta"),
			major_version: 17,
		}
	}
}

// libraries
#[derive(Debug, Deserialize, Serialize)]
pub struct Library {
	pub downloads: LibraryDownloads,
	pub name: String,
	pub rules: Option<Vec<RuleItem>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LibraryDownloads {
	pub artifact: Option<Download>,
	pub classifiers: Option<Classifiers>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Classifiers {
	#[serde(alias = "natives-linux")]
	pub natives_linux: Option<Download>,
	#[serde(alias = "natives-macos")]
	pub natives_macos: Option<Download>,
	#[serde(alias = "natives-windows")]
	pub natives_windows: Option<Download>,
	pub sources: Option<Download>,
}

// logging
#[derive(Debug, Deserialize, Serialize)]
pub struct Logging {
	pub client: LoggingClient,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoggingClient {
	pub argument: String,
	pub file: Download,
	#[serde(alias = "type")]
	pub logging_type: String,
}
