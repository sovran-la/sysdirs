//! Unix fallback platform implementation (FreeBSD, etc.)
//!
//! Uses XDG conventions similar to Linux.

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
	env_value.and_then(|val| expand_tilde_with_home(&val, home.as_deref()))
}

pub fn state_dir() -> Option<PathBuf> {
	xdg_dir("XDG_STATE_HOME", ".local/state")
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

	#[test]
	fn test_tilde_expansion_basic() {
		let home = Path::new("/home/testuser");
		let result = expand_tilde_with_home("~/my-cache", Some(home));
		assert_eq!(result, Some(PathBuf::from("/home/testuser/my-cache")));
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
	fn test_xdg_dir_fallback() {
		let home = Path::new("/home/testuser");
		let result = resolve_xdg_dir(None, Some(home), ".cache");
		assert_eq!(result, Some(PathBuf::from("/home/testuser/.cache")));
	}

	#[test]
	fn test_xdg_dir_with_tilde() {
		let home = Path::new("/home/testuser");
		let result = resolve_xdg_dir(Some("~/custom"), Some(home), ".cache");
		assert_eq!(result, Some(PathBuf::from("/home/testuser/custom")));
	}
}
