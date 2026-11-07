//! Unix fallback platform implementation (FreeBSD, etc.)
//!
//! Uses XDG Base Directory conventions. User directories (music, desktop, ...)
//! are not resolved on generic Unix. All resolution logic lives in [`crate::xdg`].

use crate::xdg;
use std::path::PathBuf;

// =============================================================================
// Directory implementations
// =============================================================================

pub fn home_dir() -> Option<PathBuf> {
	xdg::home()
}

pub fn cache_dir() -> Option<PathBuf> {
	xdg::base_dir("XDG_CACHE_HOME", ".cache")
}

pub fn config_dir() -> Option<PathBuf> {
	xdg::base_dir("XDG_CONFIG_HOME", ".config")
}

pub fn config_local_dir() -> Option<PathBuf> {
	config_dir()
}

pub fn data_dir() -> Option<PathBuf> {
	xdg::base_dir("XDG_DATA_HOME", ".local/share")
}

pub fn data_local_dir() -> Option<PathBuf> {
	data_dir()
}

pub fn executable_dir() -> Option<PathBuf> {
	xdg::base_dir("XDG_BIN_HOME", ".local/bin")
}

pub fn preference_dir() -> Option<PathBuf> {
	config_dir()
}

pub fn runtime_dir() -> Option<PathBuf> {
	xdg::optional_dir("XDG_RUNTIME_DIR")
}

pub fn state_dir() -> Option<PathBuf> {
	xdg::base_dir("XDG_STATE_HOME", ".local/state")
}

pub fn audio_dir() -> Option<PathBuf> {
	None
}

pub fn desktop_dir() -> Option<PathBuf> {
	None
}

pub fn document_dir() -> Option<PathBuf> {
	None
}

pub fn download_dir() -> Option<PathBuf> {
	None
}

pub fn font_dir() -> Option<PathBuf> {
	data_dir().map(|d| d.join("fonts"))
}

pub fn picture_dir() -> Option<PathBuf> {
	None
}

pub fn public_dir() -> Option<PathBuf> {
	None
}

pub fn template_dir() -> Option<PathBuf> {
	None
}

pub fn video_dir() -> Option<PathBuf> {
	None
}

// =============================================================================
// sysdirs extensions
// =============================================================================

pub fn temp_dir() -> Option<PathBuf> {
	xdg::temp_dir()
}

pub fn library_dir() -> Option<PathBuf> {
	None
}
