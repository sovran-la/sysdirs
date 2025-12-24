//! Integration tests for FreeBSD/Unix directory functions.
//!
//! These tests verify the public API works without manipulating env vars.
//! The core logic is tested via unit tests in src/unix.rs using dependency injection.

#![cfg(all(
	unix,
	not(target_os = "linux"),
	not(target_os = "macos"),
	not(target_os = "ios"),
	not(target_os = "android")
))]

#[test]
fn test_home_dir_returns_something() {
	let home = sysdirs::home_dir();
	assert!(home.is_some(), "home_dir() should return Some on Unix");
}

#[test]
fn test_cache_dir_returns_something() {
	let cache = sysdirs::cache_dir();
	assert!(cache.is_some(), "cache_dir() should return Some on Unix");

	let path = cache.unwrap();
	assert!(path.is_absolute(), "cache_dir should be absolute");
}

#[test]
fn test_config_dir_returns_something() {
	let config = sysdirs::config_dir();
	assert!(config.is_some(), "config_dir() should return Some on Unix");
}

#[test]
fn test_data_dir_returns_something() {
	let data = sysdirs::data_dir();
	assert!(data.is_some(), "data_dir() should return Some on Unix");
}

#[test]
fn test_temp_dir_fallback() {
	let temp = sysdirs::temp_dir();
	assert!(temp.is_some(), "temp_dir() should return Some on Unix");
}

#[test]
fn test_library_dir_none_on_unix() {
	// library_dir is Apple-only
	assert_eq!(sysdirs::library_dir(), None);
}
