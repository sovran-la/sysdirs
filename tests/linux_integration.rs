//! Integration tests for Linux directory functions.
//!
//! These tests verify the public API works without manipulating env vars.
//! The core logic (tilde expansion, XDG resolution) is tested via unit tests
//! in src/linux.rs using dependency injection.

#![cfg(target_os = "linux")]

#[test]
fn test_home_dir_returns_something() {
	// On a real Linux system, HOME should be set
	let home = sysdirs::home_dir();
	assert!(home.is_some(), "home_dir() should return Some on Linux");
}

#[test]
fn test_cache_dir_under_home_or_custom() {
	let cache = sysdirs::cache_dir();
	assert!(cache.is_some(), "cache_dir() should return Some on Linux");

	// Should either be under $HOME or a custom XDG path
	let path = cache.unwrap();
	assert!(path.is_absolute(), "cache_dir should be absolute");
}

#[test]
fn test_config_dir_returns_something() {
	let config = sysdirs::config_dir();
	assert!(config.is_some(), "config_dir() should return Some on Linux");
}

#[test]
fn test_data_dir_returns_something() {
	let data = sysdirs::data_dir();
	assert!(data.is_some(), "data_dir() should return Some on Linux");
}

#[test]
fn test_temp_dir_exists() {
	let temp = sysdirs::temp_dir();
	assert!(temp.is_some(), "temp_dir() should return Some on Linux");

	// /tmp or $TMPDIR should actually exist
	let path = temp.unwrap();
	assert!(path.exists(), "temp_dir should point to existing directory");
}

#[test]
fn test_font_dir_derived_from_data() {
	let data = sysdirs::data_dir();
	let font = sysdirs::font_dir();

	if let (Some(data_path), Some(font_path)) = (data, font) {
		assert_eq!(font_path, data_path.join("fonts"));
	}
}

#[test]
fn test_library_dir_none_on_linux() {
	// library_dir is Apple-only
	assert_eq!(sysdirs::library_dir(), None);
}
