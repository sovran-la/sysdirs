//! Tests for Android initialization.
//!
//! Verifies that init_android() and init_android_with_cache() set paths correctly.

#![cfg(target_os = "android")]

use std::path::PathBuf;

#[test]
fn test_init_android_sets_paths() {
	sysdirs::init_android("/data/data/com.example.app/files");

	assert_eq!(
		sysdirs::home_dir(),
		Some(PathBuf::from("/data/data/com.example.app/files"))
	);
	assert_eq!(
		sysdirs::config_dir(),
		Some(PathBuf::from("/data/data/com.example.app/files"))
	);
	assert_eq!(
		sysdirs::data_dir(),
		Some(PathBuf::from("/data/data/com.example.app/files"))
	);
	// Default cache is files_dir + "/cache"
	assert_eq!(
		sysdirs::cache_dir(),
		Some(PathBuf::from("/data/data/com.example.app/files/cache"))
	);
}

#[test]
fn test_init_android_with_cache_sets_separate_cache() {
	sysdirs::init_android_with_cache(
		"/data/data/com.example.app/files",
		"/data/data/com.example.app/cache",
	);

	assert_eq!(
		sysdirs::home_dir(),
		Some(PathBuf::from("/data/data/com.example.app/files"))
	);
	// Cache should be the explicit path, not derived
	assert_eq!(
		sysdirs::cache_dir(),
		Some(PathBuf::from("/data/data/com.example.app/cache"))
	);
}

#[test]
fn test_temp_dir_derived_from_files() {
	sysdirs::init_android("/data/data/com.example.app/files");

	assert_eq!(
		sysdirs::temp_dir(),
		Some(PathBuf::from("/data/data/com.example.app/files/tmp"))
	);
}

#[test]
fn test_user_dirs_return_none_on_android() {
	sysdirs::init_android("/data/data/com.example.app/files");

	// Android doesn't expose user directories to native code
	assert_eq!(sysdirs::audio_dir(), None);
	assert_eq!(sysdirs::desktop_dir(), None);
	assert_eq!(sysdirs::document_dir(), None);
	assert_eq!(sysdirs::download_dir(), None);
	assert_eq!(sysdirs::picture_dir(), None);
	assert_eq!(sysdirs::public_dir(), None);
	assert_eq!(sysdirs::video_dir(), None);
}

#[test]
fn test_library_dir_none_on_android() {
	sysdirs::init_android("/data/data/com.example.app/files");

	// Library is an Apple concept
	assert_eq!(sysdirs::library_dir(), None);
}
