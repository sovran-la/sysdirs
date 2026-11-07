//! Integration tests for Windows directory functions.
//!
//! These run on a real Windows session (CI's windows-latest), so they assert
//! shape and consistency rather than exact paths: the user name, drive letter
//! and any folder redirection are the machine's business.

#![cfg(target_os = "windows")]

use std::path::{Path, PathBuf};

fn must(name: &str, dir: Option<PathBuf>) -> PathBuf {
	let dir = dir.unwrap_or_else(|| panic!("{name}() returned None"));
	assert!(dir.is_absolute(), "{name}() not absolute: {dir:?}");
	assert!(dir.exists(), "{name}() does not exist: {dir:?}");
	assert!(dir.is_dir(), "{name}() is not a directory: {dir:?}");
	dir
}

fn ends_with_ci(path: &Path, suffix: &str) -> bool {
	path.to_string_lossy()
		.to_ascii_lowercase()
		.replace('/', "\\")
		.ends_with(&suffix.to_ascii_lowercase())
}

#[test]
fn base_dirs_resolve_and_exist() {
	must("home_dir", sysdirs::home_dir());
	must("cache_dir", sysdirs::cache_dir());
	must("config_dir", sysdirs::config_dir());
	must("config_local_dir", sysdirs::config_local_dir());
	must("data_dir", sysdirs::data_dir());
	must("data_local_dir", sysdirs::data_local_dir());
	must("preference_dir", sysdirs::preference_dir());
}

#[test]
fn appdata_dirs_have_expected_shape() {
	let cache = must("cache_dir", sysdirs::cache_dir());
	let config = must("config_dir", sysdirs::config_dir());
	assert!(ends_with_ci(&cache, "\\AppData\\Local"), "{cache:?}");
	assert!(ends_with_ci(&config, "\\AppData\\Roaming"), "{config:?}");
}

#[test]
fn local_and_roaming_are_consistent() {
	assert_eq!(sysdirs::cache_dir(), sysdirs::config_local_dir());
	assert_eq!(sysdirs::cache_dir(), sysdirs::data_local_dir());
	assert_eq!(sysdirs::config_dir(), sysdirs::data_dir());
	assert_eq!(sysdirs::config_dir(), sysdirs::preference_dir());
	assert_ne!(sysdirs::cache_dir(), sysdirs::config_dir());
}

#[test]
fn known_folders_agree_with_environment_when_not_redirected() {
	// USERPROFILE / APPDATA / LOCALAPPDATA are set by the logon process from
	// the same Known Folders, so on an unredirected machine they must match.
	// (If a CI image ever redirects these, this is the test to revisit.)
	let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
	assert_eq!(sysdirs::home_dir(), env("USERPROFILE"));
	assert_eq!(sysdirs::config_dir(), env("APPDATA"));
	assert_eq!(sysdirs::cache_dir(), env("LOCALAPPDATA"));
}

#[test]
fn user_media_dirs_resolve() {
	must("desktop_dir", sysdirs::desktop_dir());
	must("document_dir", sysdirs::document_dir());
	must("download_dir", sysdirs::download_dir());
	must("audio_dir", sysdirs::audio_dir());
	must("picture_dir", sysdirs::picture_dir());
	must("video_dir", sysdirs::video_dir());
	must("public_dir", sysdirs::public_dir());
	must("template_dir", sysdirs::template_dir());
}

#[test]
fn public_dir_is_not_under_profile() {
	let public = must("public_dir", sysdirs::public_dir());
	let home = must("home_dir", sysdirs::home_dir());
	assert!(!public.starts_with(&home), "{public:?} under {home:?}");
}

#[test]
fn template_dir_is_under_roaming_appdata() {
	let templates = must("template_dir", sysdirs::template_dir());
	let roaming = must("config_dir", sysdirs::config_dir());
	assert!(
		templates.starts_with(&roaming),
		"{templates:?} not under {roaming:?}"
	);
}

#[test]
fn temp_dir_resolves_and_exists() {
	must("temp_dir", sysdirs::temp_dir());
}

#[test]
fn unsupported_dirs_are_none() {
	assert_eq!(sysdirs::executable_dir(), None);
	assert_eq!(sysdirs::runtime_dir(), None);
	assert_eq!(sysdirs::state_dir(), None);
	assert_eq!(sysdirs::font_dir(), None);
	assert_eq!(sysdirs::library_dir(), None);
}
