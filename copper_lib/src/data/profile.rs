use serde::{Deserialize, Serialize};

/// A profile for a specific version of Minecraft. Contains arguments to pass to Minecraft & Java, dependancies, download URLs, and other data.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Profile {
	#[serde(alias = "minecraftArguments")]
	pub arguments: Arguments,
	#[serde(alias = "assetIndex")]
	pub asset_index: Download,
	pub assets: String,
	#[serde(alias = "complianceLevel", default)]
	pub compliance_level: i32,
	pub downloads: Downloads,
	pub id: String,
	#[serde(alias = "javaVersion", default)]
	pub java_version: JavaVersion,
	pub libraries: Vec<Library>,
	#[serde(skip_serializing_if = "Option::is_none")]
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

/// An array of rules specifying whether a certain value should be used or not. If there is more than one rule in the array, all of them must be true for the value to be used.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Rule {
	pub rules: Vec<RuleItem>,
	pub value: RuleValue,
}

/// An item in the array of rules contained in [Rule].
/// If `action` is `disallow`, only true if the features/os are anything but the value.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RuleItem {
	pub action: RuleAction,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub features: Option<RuleItemFeatures>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub os: Option<RuleItemOs>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
	Allow,
	Disallow,
}

/// `is_demo_user`: If the Minecraft is running in demo mode.
/// `has_custom_resolution`: If a custom resolution is being passed to Minecraft.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RuleItemFeatures {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub is_demo_user: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub has_custom_resolution: Option<bool>,
}

/// `arch`: Architecture of CPU. Observed values: `x86`.
/// `name`: Name of OS. Observed values: `osx`, `windows`, `linux`.
/// `version`: Version of OS. Seems to only be valid for Windows. Observed values: `^10\\.`
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RuleItemOs {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub arch: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RuleValue {
	String(String),
	Vec(Vec<String>),
}

/// A download for a jar, library, etc.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Download {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub path: Option<String>,
	pub sha1: String,
	pub size: i32,
	#[serde(alias = "totalSize", skip_serializing_if = "Option::is_none")]
	pub total_size: Option<i32>,
	pub url: String,
}

/// Either the old or new format for arguments to pass to Minecraft. The new version contains an array, and the old version is a string with elements seperated by spaces.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Arguments {
	NewArguments(NewArguments),
	OldArguments(String),
}

/// Arguments to pass to Minecraft and Java.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewArguments {
	pub game: Vec<Argument>,
	pub jvm: Vec<Argument>,
}

/// An individual argument (to Minecraft or Java). Can either be a String that's always true, or a Rule that's conditional.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Argument {
	String(String),
	Rule(Rule),
}

/// Downloads for the client and server of a specific Minecraft version, as well as mappings.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Downloads {
	pub client: Download,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub client_mappings: Option<Download>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub server: Option<Download>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub server_mappings: Option<Download>,
}

/// The version of Java to launch Minecraft with. Only specifies the major version of Java to use.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JavaVersion {
	pub component: String,
	#[serde(alias = "majorVersion")]
	pub major_version: i32,
}
impl Default for JavaVersion {
	fn default() -> Self {
		Self {
			component: "java-runtime-beta".into(),
			major_version: 17,
		}
	}
}

/// A library that needs to be downloaded and added to the classpath to launch Minecraft.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Library {
	pub downloads: LibraryDownloads,
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub rules: Option<Vec<RuleItem>>,
}

/// Downloads for a library's jar and classifiers (native components of libraries)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LibraryDownloads {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub artifact: Option<Download>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub classifiers: Option<Classifiers>,
}

/// Downloads for classifiers (native components) of libraries.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Classifiers {
	#[serde(alias = "natives-linux", skip_serializing_if = "Option::is_none")]
	pub natives_linux: Option<Download>,
	#[serde(alias = "natives-macos", skip_serializing_if = "Option::is_none")]
	pub natives_macos: Option<Download>,
	#[serde(alias = "natives-windows", skip_serializing_if = "Option::is_none")]
	pub natives_windows: Option<Download>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub sources: Option<Download>,
}

/// Which logging client is used by this version.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Logging {
	pub client: LoggingClient,
}

/// Info about the logging client this version uses and sometimes flags that should be passed to Java.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoggingClient {
	pub argument: String,
	pub file: Download,
	#[serde(alias = "type")]
	pub logging_type: String,
}
