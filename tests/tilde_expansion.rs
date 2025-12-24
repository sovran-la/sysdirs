//! Tests for tilde expansion in paths.
//!
//! Verifies that paths like "~/cache" get expanded to "/home/user/cache".
//! This handles users who set XDG vars with tildes (against spec, but common).

#![cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]

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
fn test_tilde_expansion_cache_dir() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_CACHE_HOME", Some("~/my-cache")),
		],
		|| {
			let cache = sysdirs::cache_dir();
			assert_eq!(cache, Some(PathBuf::from("/home/testuser/my-cache")));
		},
	);
}

#[test]
fn test_tilde_expansion_config_dir() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_CONFIG_HOME", Some("~/my-config")),
		],
		|| {
			let config = sysdirs::config_dir();
			assert_eq!(config, Some(PathBuf::from("/home/testuser/my-config")));
		},
	);
}

#[test]
fn test_tilde_expansion_data_dir() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_DATA_HOME", Some("~/my-data")),
		],
		|| {
			let data = sysdirs::data_dir();
			assert_eq!(data, Some(PathBuf::from("/home/testuser/my-data")));
		},
	);
}

#[test]
fn test_tilde_expansion_state_dir() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_STATE_HOME", Some("~/my-state")),
		],
		|| {
			let state = sysdirs::state_dir();
			assert_eq!(state, Some(PathBuf::from("/home/testuser/my-state")));
		},
	);
}

#[test]
fn test_tilde_expansion_runtime_dir() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_RUNTIME_DIR", Some("~/my-runtime")),
		],
		|| {
			let runtime = sysdirs::runtime_dir();
			assert_eq!(runtime, Some(PathBuf::from("/home/testuser/my-runtime")));
		},
	);
}

#[test]
fn test_tilde_expansion_executable_dir() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_BIN_HOME", Some("~/my-bin")),
		],
		|| {
			let bin = sysdirs::executable_dir();
			assert_eq!(bin, Some(PathBuf::from("/home/testuser/my-bin")));
		},
	);
}

#[test]
fn test_tilde_only_expands_to_home() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_CACHE_HOME", Some("~")),
		],
		|| {
			let cache = sysdirs::cache_dir();
			assert_eq!(cache, Some(PathBuf::from("/home/testuser")));
		},
	);
}

#[test]
fn test_no_expansion_for_absolute_path() {
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_CACHE_HOME", Some("/absolute/path/cache")),
		],
		|| {
			let cache = sysdirs::cache_dir();
			assert_eq!(cache, Some(PathBuf::from("/absolute/path/cache")));
		},
	);
}

#[test]
fn test_no_expansion_for_tilde_in_middle() {
	// Tilde in the middle of path should NOT expand (that's not how shells work)
	with_env(
		&[
			("HOME", Some("/home/testuser")),
			("XDG_CACHE_HOME", Some("/some/~/path")),
		],
		|| {
			let cache = sysdirs::cache_dir();
			assert_eq!(cache, Some(PathBuf::from("/some/~/path")));
		},
	);
}

#[test]
fn test_tilde_expansion_with_missing_home() {
	with_env(
		&[
			("HOME", None),
			("XDG_CACHE_HOME", Some("~/my-cache")),
		],
		|| {
			// Can't expand ~ without HOME, should return None
			let cache = sysdirs::cache_dir();
			assert_eq!(cache, None);
		},
	);
}
