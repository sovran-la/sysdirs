//! Tests for Android initialization.
//!
//! Initialization is process-wide and happens once, so every assertion that
//! depends on state lives in a single test function with a fixed order. Two
//! `#[test]`s that both call `init_android` would race for the `OnceLock`,
//! and whichever lost would fail nondeterministically.

#![cfg(target_os = "android")]

use std::path::PathBuf;
use sysdirs::AndroidInitError;

const FILES: &str = "/data/data/com.example.app/files";

#[test]
fn init_lifecycle() {
	// --- Before init: nothing is available ---------------------------------
	assert_eq!(sysdirs::home_dir(), None);
	assert_eq!(sysdirs::cache_dir(), None);
	assert_eq!(sysdirs::config_dir(), None);
	assert_eq!(sysdirs::data_dir(), None);
	assert_eq!(sysdirs::temp_dir(), None);

	// --- Invalid paths are rejected and leave the state untouched ----------
	assert_eq!(
		sysdirs::init_android(""),
		Err(AndroidInitError::InvalidPath)
	);
	assert_eq!(
		sysdirs::init_android("files"),
		Err(AndroidInitError::InvalidPath)
	);
	assert_eq!(
		sysdirs::init_android_with_cache(FILES, "cache"),
		Err(AndroidInitError::InvalidPath)
	);
	assert_eq!(
		sysdirs::init_android_with_cache("", "/data/data/com.example.app/cache"),
		Err(AndroidInitError::InvalidPath)
	);
	assert_eq!(
		sysdirs::home_dir(),
		None,
		"rejected init must not set paths"
	);
	assert_eq!(
		sysdirs::cache_dir(),
		None,
		"rejected init must not set paths"
	);

	// --- First valid init wins ---------------------------------------------
	assert_eq!(sysdirs::init_android(FILES), Ok(()));

	let files = PathBuf::from(FILES);
	assert_eq!(sysdirs::home_dir(), Some(files.clone()));
	assert_eq!(sysdirs::config_dir(), Some(files.clone()));
	assert_eq!(sysdirs::config_local_dir(), Some(files.clone()));
	assert_eq!(sysdirs::data_dir(), Some(files.clone()));
	assert_eq!(sysdirs::data_local_dir(), Some(files.clone()));
	assert_eq!(sysdirs::preference_dir(), Some(files.clone()));
	assert_eq!(sysdirs::cache_dir(), Some(files.join("cache")));
	assert_eq!(sysdirs::temp_dir(), Some(files.join("tmp")));

	// --- Second init is refused and changes nothing ------------------------
	assert_eq!(
		sysdirs::init_android("/data/data/other.app/files"),
		Err(AndroidInitError::AlreadyInitialized)
	);
	assert_eq!(
		sysdirs::init_android_with_cache(
			"/data/data/other.app/files",
			"/data/data/other.app/cache"
		),
		Err(AndroidInitError::AlreadyInitialized)
	);
	assert_eq!(sysdirs::home_dir(), Some(files.clone()));
	assert_eq!(sysdirs::cache_dir(), Some(files.join("cache")));

	// --- Lookups are stable across repeated calls --------------------------
	for _ in 0..3 {
		assert_eq!(sysdirs::data_dir(), Some(files.clone()));
	}

	// --- Things Android never exposes to native code -----------------------
	assert_eq!(sysdirs::executable_dir(), None);
	assert_eq!(sysdirs::runtime_dir(), None);
	assert_eq!(sysdirs::state_dir(), None);
	assert_eq!(sysdirs::audio_dir(), None);
	assert_eq!(sysdirs::desktop_dir(), None);
	assert_eq!(sysdirs::document_dir(), None);
	assert_eq!(sysdirs::download_dir(), None);
	assert_eq!(sysdirs::font_dir(), None);
	assert_eq!(sysdirs::picture_dir(), None);
	assert_eq!(sysdirs::public_dir(), None);
	assert_eq!(sysdirs::template_dir(), None);
	assert_eq!(sysdirs::video_dir(), None);
	assert_eq!(sysdirs::library_dir(), None);
}

#[test]
fn error_is_std_error() {
	fn assert_error<E: std::error::Error + Send + Sync + 'static>() {}
	assert_error::<AndroidInitError>();
}
