//! Tests for XDG fallback behavior.
//!
//! Verifies that when XDG env vars are unset, we fall back to the correct defaults.

#![cfg(target_os = "linux")]

use std::env;
use std::path::PathBuf;

/// Helper to run a test with temporary env var changes, restoring afterwards.
///
/// # Safety
/// This modifies environment variables which is inherently unsafe in multi-threaded
/// contexts. Tests using this helper should run with --test-threads=1 or accept
/// potential flakiness.
fn with_env<F, R>(vars: &[(&str, Option<&str>)], f: F) -> R
where
	F: FnOnce() -> R,
{
	// Save original values
	let originals: Vec<_> = vars
		.iter()
		.map(|(k, _)| (*k, env::var_os(k)))
		.collect();

	// Set new values
	for (k, v) in vars {
		// SAFETY: We're in a test context and accept the risk of env var mutation.
		// These tests are cfg-gated to specific platforms and don't run in parallel
		// with production code.
		unsafe {
			match v {
				Some(val) => env::set_var(k, val),
				None => env::remove_var(k),
			}
		}
	}

	let result = f();

	// Restore original values
	for (k, original) in originals {
		// SAFETY: Same as above - restoring original env state.
		unsafe {
			match original {
				Some(val) => env::set_var(k, val),
				None => env::remove_var(k),
			}
		}
	}

	result
}

#[test]
fn test_cache_dir_fallback() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_CACHE_HOME", None),
		],
		|| {
			let cache = sysdirs::cache_dir();
			assert_eq!(cache, Some(PathBuf::from("/home/testuser/.cache")));
		},
	);
}

#[test]
fn test_config_dir_fallback() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_CONFIG_HOME", None),
		],
		|| {
			let config = sysdirs::config_dir();
			assert_eq!(config, Some(PathBuf::from("/home/testuser/.config")));
		},
	);
}

#[test]
fn test_data_dir_fallback() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_DATA_HOME", None),
		],
		|| {
			let data = sysdirs::data_dir();
			assert_eq!(data, Some(PathBuf::from("/home/testuser/.local/share")));
		},
	);
}

#[test]
fn test_state_dir_fallback() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_STATE_HOME", None),
		],
		|| {
			let state = sysdirs::state_dir();
			assert_eq!(state, Some(PathBuf::from("/home/testuser/.local/state")));
		},
	);
}

#[test]
fn test_executable_dir_fallback() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_BIN_HOME", None),
		],
		|| {
			let bin = sysdirs::executable_dir();
			assert_eq!(bin, Some(PathBuf::from("/home/testuser/.local/bin")));
		},
	);
}

#[test]
fn test_runtime_dir_no_fallback() {
	// XDG_RUNTIME_DIR has no default - it should return None if unset
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_RUNTIME_DIR", None),
		],
		|| {
			let runtime = sysdirs::runtime_dir();
			assert_eq!(runtime, None);
		},
	);
}

#[test]
fn test_temp_dir_fallback() {
	with_env(
		&[("TMPDIR", None)],
		|| {
			let temp = sysdirs::temp_dir();
			assert_eq!(temp, Some(PathBuf::from("/tmp")));
		},
	);
}

#[test]
fn test_user_dirs_no_fallback() {
	// XDG user directories have no defaults - they should return None if unset
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_MUSIC_DIR", None),
			("XDG_DESKTOP_DIR", None),
			("XDG_DOCUMENTS_DIR", None),
			("XDG_DOWNLOAD_DIR", None),
			("XDG_PICTURES_DIR", None),
			("XDG_PUBLICSHARE_DIR", None),
			("XDG_TEMPLATES_DIR", None),
			("XDG_VIDEOS_DIR", None),
		],
		|| {
			assert_eq!(sysdirs::audio_dir(), None);
			assert_eq!(sysdirs::desktop_dir(), None);
			assert_eq!(sysdirs::document_dir(), None);
			assert_eq!(sysdirs::download_dir(), None);
			assert_eq!(sysdirs::picture_dir(), None);
			assert_eq!(sysdirs::public_dir(), None);
			assert_eq!(sysdirs::template_dir(), None);
			assert_eq!(sysdirs::video_dir(), None);
		},
	);
}

#[test]
fn test_font_dir_derived_from_data() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_DATA_HOME", None),
		],
		|| {
			let font = sysdirs::font_dir();
			assert_eq!(font, Some(PathBuf::from("/home/testuser/.local/share/fonts")));
		},
	);
}

#[test]
fn test_font_dir_follows_custom_data_dir() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_DATA_HOME", Some("/custom/data")),
		],
		|| {
			let font = sysdirs::font_dir();
			assert_eq!(font, Some(PathBuf::from("/custom/data/fonts")));
		},
	);
}

#[test]
fn test_all_dirs_none_without_home() {
	with_env(
		&[
			("HOME", None),
			("XDG_CACHE_HOME", None),
			("XDG_CONFIG_HOME", None),
			("XDG_DATA_HOME", None),
			("XDG_STATE_HOME", None),
			("XDG_BIN_HOME", None),
		],
		|| {
			assert_eq!(sysdirs::home_dir(), None);
			assert_eq!(sysdirs::cache_dir(), None);
			assert_eq!(sysdirs::config_dir(), None);
			assert_eq!(sysdirs::data_dir(), None);
			assert_eq!(sysdirs::state_dir(), None);
			assert_eq!(sysdirs::executable_dir(), None);
		},
	);
}
