//! Windows platform implementation
//!
//! TODO: Implement using Known Folders API for full correctness.
//! Currently uses environment variables as fallback.

use std::path::PathBuf;

// =============================================================================
// Helpers
// =============================================================================

fn home() -> Option<PathBuf> {
	std::env::var_os("USERPROFILE").map(PathBuf::from)
}

fn appdata_roaming() -> Option<PathBuf> {
	std::env::var_os("APPDATA").map(PathBuf::from)
}

fn appdata_local() -> Option<PathBuf> {
	std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

// =============================================================================
// Directory implementations
// =============================================================================

pub fn home_dir() -> Option<PathBuf> {
	home()
}

pub fn cache_dir() -> Option<PathBuf> {
	appdata_local()
}

pub fn config_dir() -> Option<PathBuf> {
	appdata_roaming()
}

pub fn config_local_dir() -> Option<PathBuf> {
	appdata_local()
}

pub fn data_dir() -> Option<PathBuf> {
	appdata_roaming()
}

pub fn data_local_dir() -> Option<PathBuf> {
	appdata_local()
}

pub fn executable_dir() -> Option<PathBuf> {
	None
}

pub fn preference_dir() -> Option<PathBuf> {
	appdata_roaming()
}

pub fn runtime_dir() -> Option<PathBuf> {
	None
}

pub fn state_dir() -> Option<PathBuf> {
	None
}

pub fn audio_dir() -> Option<PathBuf> {
	home().map(|h| h.join("Music"))
}

pub fn desktop_dir() -> Option<PathBuf> {
	home().map(|h| h.join("Desktop"))
}

pub fn document_dir() -> Option<PathBuf> {
	home().map(|h| h.join("Documents"))
}

pub fn download_dir() -> Option<PathBuf> {
	home().map(|h| h.join("Downloads"))
}

pub fn font_dir() -> Option<PathBuf> {
	None
}

pub fn picture_dir() -> Option<PathBuf> {
	home().map(|h| h.join("Pictures"))
}

pub fn public_dir() -> Option<PathBuf> {
	std::env::var_os("PUBLIC").map(PathBuf::from)
}

pub fn template_dir() -> Option<PathBuf> {
	appdata_roaming().map(|a| a.join("Microsoft\\Windows\\Templates"))
}

pub fn video_dir() -> Option<PathBuf> {
	home().map(|h| h.join("Videos"))
}

// =============================================================================
// sysdirs extensions
// =============================================================================

pub fn temp_dir() -> Option<PathBuf> {
	std::env::var_os("TEMP")
		.or_else(|| std::env::var_os("TMP"))
		.map(PathBuf::from)
}

pub fn library_dir() -> Option<PathBuf> {
	None
}
