//! Linux platform implementation
//!
//! Uses XDG Base Directory and XDG User Directory specifications.

use std::path::{Path, PathBuf};

// =============================================================================
// Core logic (testable, no env access)
// =============================================================================

/// Expand tilde in a path string given a home directory.
/// This is the testable core - no env var access.
fn expand_tilde_with_home(path_str: &str, home: Option<&Path>) -> Option<PathBuf> {
	if let Some(rest) = path_str.strip_prefix("~/") {
		home.map(|h| h.join(rest))
	} else if path_str == "~" {
		home.map(|h| h.to_path_buf())
	} else {
		Some(PathBuf::from(path_str))
	}
}

/// Resolve an XDG directory given an env value, home dir, and default suffix.
/// This is the testable core - no env var access.
fn resolve_xdg_dir(
	env_value: Option<&str>,
	home: Option<&Path>,
	default_suffix: &str,
) -> Option<PathBuf> {
	match env_value {
		Some(val) => expand_tilde_with_home(val, home),
		None => home.map(|h| h.join(default_suffix)),
	}
}

/// Resolve an XDG user directory (no default fallback).
fn resolve_xdg_user_dir(env_value: Option<&str>, home: Option<&Path>) -> Option<PathBuf> {
	env_value.and_then(|val| expand_tilde_with_home(val, home))
}

// =============================================================================
// Env var wrappers
// =============================================================================

fn home() -> Option<PathBuf> {
	std::env::var_os("HOME").map(PathBuf::from)
}

fn home_ref() -> Option<PathBuf> {
	home()
}

fn xdg_dir(env_var: &str, default_suffix: &str) -> Option<PathBuf> {
	let home = home_ref();
	let env_value = std::env::var(env_var).ok();
	resolve_xdg_dir(env_value.as_deref(), home.as_deref(), default_suffix)
}

fn xdg_user_dir(env_var: &str) -> Option<PathBuf> {
	let home = home_ref();
	let env_value = std::env::var(env_var).ok();
	resolve_xdg_user_dir(env_value.as_deref(), home.as_deref())
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
	let home = home_ref();
	let env_value = std::env::var("XDG_BIN_HOME").ok();
	resolve_xdg_dir(env_value.as_deref(), home.as_deref(), ".local/bin")
}

pub fn preference_dir() -> Option<PathBuf> {
	config_dir()
}

pub fn runtime_dir() -> Option<PathBuf> {
	let home = home_ref();
	let env_value = std::env::var("XDG_RUNTIME_DIR").ok();
	resolve_xdg_user_dir(env_value.as_deref(), home.as_deref())
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
	let home = home_ref();
	let env_value = std::env::var("TMPDIR").ok();
	match env_value.as_deref() {
		Some(val) => expand_tilde_with_home(val, home.as_deref()),
		None => Some(PathBuf::from("/tmp")),
	}
}

pub fn library_dir() -> Option<PathBuf> {
	None
}

// =============================================================================
// Tests (parallel-safe, no env manipulation)
// =============================================================================

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::Path;

	// -------------------------------------------------------------------------
	// Tilde expansion tests
	// -------------------------------------------------------------------------

	#[test]
	fn test_tilde_expansion_basic() {
		let home = Path::new("/home/testuser");
		let result = expand_tilde_with_home("~/my-cache", Some(home));
		assert_eq!(result, Some(PathBuf::from("/home/testuser/my-cache")));
	}

	#[test]
	fn test_tilde_expansion_nested() {
		let home = Path::new("/home/testuser");
		let result = expand_tilde_with_home("~/foo/bar/baz", Some(home));
		assert_eq!(result, Some(PathBuf::from("/home/testuser/foo/bar/baz")));
	}

	#[test]
	fn test_tilde_only() {
		let home = Path::new("/home/testuser");
		let result = expand_tilde_with_home("~", Some(home));
		assert_eq!(result, Some(PathBuf::from("/home/testuser")));
	}

	#[test]
	fn test_absolute_path_unchanged() {
		let home = Path::new("/home/testuser");
		let result = expand_tilde_with_home("/absolute/path", Some(home));
		assert_eq!(result, Some(PathBuf::from("/absolute/path")));
	}

	#[test]
	fn test_tilde_in_middle_unchanged() {
		let home = Path::new("/home/testuser");
		let result = expand_tilde_with_home("/some/~/path", Some(home));
		assert_eq!(result, Some(PathBuf::from("/some/~/path")));
	}

	#[test]
	fn test_tilde_expansion_no_home() {
		let result = expand_tilde_with_home("~/my-cache", None);
		assert_eq!(result, None);
	}

	#[test]
	fn test_tilde_only_no_home() {
		let result = expand_tilde_with_home("~", None);
		assert_eq!(result, None);
	}

	#[test]
	fn test_absolute_path_no_home() {
		// Absolute paths should work even without home
		let result = expand_tilde_with_home("/absolute/path", None);
		assert_eq!(result, Some(PathBuf::from("/absolute/path")));
	}

	// -------------------------------------------------------------------------
	// XDG resolution tests
	// -------------------------------------------------------------------------

	#[test]
	fn test_xdg_dir_with_env_value() {
		let home = Path::new("/home/testuser");
		let result = resolve_xdg_dir(Some("/custom/cache"), Some(home), ".cache");
		assert_eq!(result, Some(PathBuf::from("/custom/cache")));
	}

	#[test]
	fn test_xdg_dir_with_tilde_env_value() {
		let home = Path::new("/home/testuser");
		let result = resolve_xdg_dir(Some("~/my-cache"), Some(home), ".cache");
		assert_eq!(result, Some(PathBuf::from("/home/testuser/my-cache")));
	}

	#[test]
	fn test_xdg_dir_fallback_to_default() {
		let home = Path::new("/home/testuser");
		let result = resolve_xdg_dir(None, Some(home), ".cache");
		assert_eq!(result, Some(PathBuf::from("/home/testuser/.cache")));
	}

	#[test]
	fn test_xdg_dir_no_home_no_env() {
		let result = resolve_xdg_dir(None, None, ".cache");
		assert_eq!(result, None);
	}

	#[test]
	fn test_xdg_user_dir_with_value() {
		let home = Path::new("/home/testuser");
		let result = resolve_xdg_user_dir(Some("/home/testuser/Music"), Some(home));
		assert_eq!(result, Some(PathBuf::from("/home/testuser/Music")));
	}

	#[test]
	fn test_xdg_user_dir_with_tilde() {
		let home = Path::new("/home/testuser");
		let result = resolve_xdg_user_dir(Some("~/Music"), Some(home));
		assert_eq!(result, Some(PathBuf::from("/home/testuser/Music")));
	}

	#[test]
	fn test_xdg_user_dir_no_value() {
		let home = Path::new("/home/testuser");
		// User dirs have no default - should return None
		let result = resolve_xdg_user_dir(None, Some(home));
		assert_eq!(result, None);
	}

	// -------------------------------------------------------------------------
	// Default path tests
	// -------------------------------------------------------------------------

	#[test]
	fn test_cache_default() {
		let home = Path::new("/home/alice");
		let result = resolve_xdg_dir(None, Some(home), ".cache");
		assert_eq!(result, Some(PathBuf::from("/home/alice/.cache")));
	}

	#[test]
	fn test_config_default() {
		let home = Path::new("/home/alice");
		let result = resolve_xdg_dir(None, Some(home), ".config");
		assert_eq!(result, Some(PathBuf::from("/home/alice/.config")));
	}

	#[test]
	fn test_data_default() {
		let home = Path::new("/home/alice");
		let result = resolve_xdg_dir(None, Some(home), ".local/share");
		assert_eq!(result, Some(PathBuf::from("/home/alice/.local/share")));
	}

	#[test]
	fn test_state_default() {
		let home = Path::new("/home/alice");
		let result = resolve_xdg_dir(None, Some(home), ".local/state");
		assert_eq!(result, Some(PathBuf::from("/home/alice/.local/state")));
	}

	#[test]
	fn test_bin_default() {
		let home = Path::new("/home/alice");
		let result = resolve_xdg_dir(None, Some(home), ".local/bin");
		assert_eq!(result, Some(PathBuf::from("/home/alice/.local/bin")));
	}
}
