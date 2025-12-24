//! Linux platform implementation
//!
//! Uses XDG Base Directory and XDG User Directory specifications.

use std::path::PathBuf;

// =============================================================================
// Helpers
// =============================================================================

fn home() -> Option<PathBuf> {
	std::env::var_os("HOME").map(PathBuf::from)
}

/// Expand tilde in paths for users who don't read specs.
fn expand_tilde(path: PathBuf) -> Option<PathBuf> {
	let path_str = path.to_str()?;
	if let Some(rest) = path_str.strip_prefix("~/") {
		home().map(|h| h.join(rest))
	} else if path_str == "~" {
		home()
	} else {
		Some(path)
	}
}

fn xdg_dir(env_var: &str, default_path: &str) -> Option<PathBuf> {
	std::env::var_os(env_var)
		.map(PathBuf::from)
		.and_then(expand_tilde)
		.or_else(|| home().map(|h| h.join(default_path)))
}

fn xdg_user_dir(env_var: &str) -> Option<PathBuf> {
	// XDG user dirs return None if not set (no defaults)
	std::env::var_os(env_var)
		.map(PathBuf::from)
		.and_then(expand_tilde)
}

// =============================================================================
// Directory implementations
// =============================================================================

pub fn home_dir() -> Option<PathBuf> {
	home()
}

pub fn cache_dir() -> Option<PathBuf> {
	xdg_dir("XDG_CACHE_HOME", ".cache")
}

pub fn config_dir() -> Option<PathBuf> {
	xdg_dir("XDG_CONFIG_HOME", ".config")
}

pub fn config_local_dir() -> Option<PathBuf> {
	config_dir()
}

pub fn data_dir() -> Option<PathBuf> {
	xdg_dir("XDG_DATA_HOME", ".local/share")
}

pub fn data_local_dir() -> Option<PathBuf> {
	data_dir()
}

pub fn executable_dir() -> Option<PathBuf> {
	std::env::var_os("XDG_BIN_HOME")
		.map(PathBuf::from)
		.and_then(expand_tilde)
		.or_else(|| home().map(|h| h.join(".local/bin")))
}

pub fn preference_dir() -> Option<PathBuf> {
	config_dir()
}

pub fn runtime_dir() -> Option<PathBuf> {
	std::env::var_os("XDG_RUNTIME_DIR")
		.map(PathBuf::from)
		.and_then(expand_tilde)
}

pub fn state_dir() -> Option<PathBuf> {
	xdg_dir("XDG_STATE_HOME", ".local/state")
}

pub fn audio_dir() -> Option<PathBuf> {
	xdg_user_dir("XDG_MUSIC_DIR")
}

pub fn desktop_dir() -> Option<PathBuf> {
	xdg_user_dir("XDG_DESKTOP_DIR")
}

pub fn document_dir() -> Option<PathBuf> {
	xdg_user_dir("XDG_DOCUMENTS_DIR")
}

pub fn download_dir() -> Option<PathBuf> {
	xdg_user_dir("XDG_DOWNLOAD_DIR")
}

pub fn font_dir() -> Option<PathBuf> {
	data_dir().map(|d| d.join("fonts"))
}

pub fn picture_dir() -> Option<PathBuf> {
	xdg_user_dir("XDG_PICTURES_DIR")
}

pub fn public_dir() -> Option<PathBuf> {
	xdg_user_dir("XDG_PUBLICSHARE_DIR")
}

pub fn template_dir() -> Option<PathBuf> {
	xdg_user_dir("XDG_TEMPLATES_DIR")
}

pub fn video_dir() -> Option<PathBuf> {
	xdg_user_dir("XDG_VIDEOS_DIR")
}

// =============================================================================
// sysdirs extensions
// =============================================================================

pub fn temp_dir() -> Option<PathBuf> {
	std::env::var_os("TMPDIR")
		.map(PathBuf::from)
		.and_then(expand_tilde)
		.or_else(|| Some(PathBuf::from("/tmp")))
}

pub fn library_dir() -> Option<PathBuf> {
	None
}
