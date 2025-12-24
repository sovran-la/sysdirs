//! Tests for Apple search path domain switching.
//!
//! Verifies that set_domain() changes the paths returned by directory functions.

#![cfg(target_os = "macos")]

use std::path::PathBuf;
use sysdirs::{SearchPathDomain, set_domain};

#[test]
fn test_user_domain_returns_home_paths() {
	set_domain(SearchPathDomain::User);

	let cache = sysdirs::cache_dir();
	assert!(cache.is_some());
	let cache_path = cache.unwrap();

	// User domain should return paths under ~/Library
	let home = std::env::var("HOME").expect("HOME not set");
	assert!(
		cache_path.starts_with(&home),
		"Expected path under {}, got {:?}",
		home,
		cache_path
	);
	assert!(
		cache_path.to_string_lossy().contains("Library/Caches"),
		"Expected Library/Caches in path, got {:?}",
		cache_path
	);
}

#[test]
fn test_local_domain_returns_root_library() {
	set_domain(SearchPathDomain::Local);

	let cache = sysdirs::cache_dir();
	assert!(cache.is_some());
	let cache_path = cache.unwrap();

	// Local domain should return /Library/Caches
	assert_eq!(
		cache_path,
		PathBuf::from("/Library/Caches"),
		"Expected /Library/Caches, got {:?}",
		cache_path
	);
}

#[test]
fn test_system_domain_returns_system_library() {
	set_domain(SearchPathDomain::System);

	let cache = sysdirs::cache_dir();
	assert!(cache.is_some());
	let cache_path = cache.unwrap();

	// System domain should return /System/Library/Caches
	assert_eq!(
		cache_path,
		PathBuf::from("/System/Library/Caches"),
		"Expected /System/Library/Caches, got {:?}",
		cache_path
	);
}

#[test]
fn test_network_domain_returns_network_library() {
	set_domain(SearchPathDomain::Network);

	let library = sysdirs::library_dir();
	assert!(library.is_some());
	let library_path = library.unwrap();

	// Network domain should return /Network/Library
	assert_eq!(
		library_path,
		PathBuf::from("/Network/Library"),
		"Expected /Network/Library, got {:?}",
		library_path
	);
}

#[test]
fn test_domain_affects_multiple_dirs() {
	set_domain(SearchPathDomain::Local);

	let cache = sysdirs::cache_dir();
	let config = sysdirs::config_dir();
	let library = sysdirs::library_dir();

	assert_eq!(cache, Some(PathBuf::from("/Library/Caches")));
	assert_eq!(config, Some(PathBuf::from("/Library/Application Support")));
	assert_eq!(library, Some(PathBuf::from("/Library")));
}

#[test]
fn test_domain_switch_is_immediate() {
	// Start with user
	set_domain(SearchPathDomain::User);
	let user_cache = sysdirs::cache_dir().unwrap();

	// Switch to local
	set_domain(SearchPathDomain::Local);
	let local_cache = sysdirs::cache_dir().unwrap();

	// Switch back to user
	set_domain(SearchPathDomain::User);
	let user_cache_again = sysdirs::cache_dir().unwrap();

	// Verify they're different
	assert_ne!(user_cache, local_cache);
	assert_eq!(user_cache, user_cache_again);
	assert_eq!(local_cache, PathBuf::from("/Library/Caches"));
}

#[test]
fn test_home_dir_ignores_domain() {
	// home_dir uses $HOME, not sysdir, so domain shouldn't matter
	let home = std::env::var("HOME").expect("HOME not set");

	set_domain(SearchPathDomain::User);
	let home_user = sysdirs::home_dir();

	set_domain(SearchPathDomain::Local);
	let home_local = sysdirs::home_dir();

	set_domain(SearchPathDomain::System);
	let home_system = sysdirs::home_dir();

	// All should be the same
	assert_eq!(home_user, Some(PathBuf::from(&home)));
	assert_eq!(home_local, Some(PathBuf::from(&home)));
	assert_eq!(home_system, Some(PathBuf::from(&home)));
}

#[test]
fn test_preference_dir_derived_from_library() {
	set_domain(SearchPathDomain::Local);

	let prefs = sysdirs::preference_dir();
	assert_eq!(prefs, Some(PathBuf::from("/Library/Preferences")));

	set_domain(SearchPathDomain::User);

	let user_prefs = sysdirs::preference_dir();
	assert!(user_prefs.is_some());
	assert!(
		user_prefs
			.unwrap()
			.to_string_lossy()
			.contains("Library/Preferences"),
		"Expected Library/Preferences in path"
	);
}

#[test]
fn test_font_dir_derived_from_library() {
	set_domain(SearchPathDomain::Local);

	let fonts = sysdirs::font_dir();
	assert_eq!(fonts, Some(PathBuf::from("/Library/Fonts")));
}
