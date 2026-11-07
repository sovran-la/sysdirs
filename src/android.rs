//! Android platform implementation
//!
//! Android apps run in a sandbox with randomized paths. This module supports two
//! initialization methods:
//!
//! 1. Manual init via `init_android()` - for Kotlin/Java apps embedding Rust
//! 2. Auto-detection via ndk-context - for pure Rust Android apps (requires `android-auto` feature)
//!
//! Both routes populate the same process-wide `OnceLock`s, so initialization
//! happens exactly once and later lookups are plain clones.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// =============================================================================
// Errors
// =============================================================================

/// Why an `init_android*` call did not take effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidInitError {
	/// A path was empty or not absolute. Nothing was changed.
	InvalidPath,
	/// Paths were already set by an earlier call (or by `android-auto`
	/// detection). The earlier values are kept.
	AlreadyInitialized,
}

impl fmt::Display for AndroidInitError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidPath => f.write_str("android path must be absolute and non-empty"),
			Self::AlreadyInitialized => f.write_str("android paths are already initialized"),
		}
	}
}

impl std::error::Error for AndroidInitError {}

// =============================================================================
// Initialization
// =============================================================================

static ANDROID_FILES_DIR: OnceLock<PathBuf> = OnceLock::new();
static ANDROID_CACHE_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Validate a caller-supplied sandbox path.
fn validate(path: &str) -> Result<PathBuf, AndroidInitError> {
	if path.is_empty() || !Path::new(path).is_absolute() {
		return Err(AndroidInitError::InvalidPath);
	}
	Ok(PathBuf::from(path))
}

/// Store both paths atomically with respect to the "already initialized" check.
///
/// `OnceLock::set` on the files dir is the gate: whoever wins it also sets the
/// cache dir. A concurrent loser gets `AlreadyInitialized` and changes nothing.
fn store(files: PathBuf, cache: PathBuf) -> Result<(), AndroidInitError> {
	ANDROID_FILES_DIR
		.set(files)
		.map_err(|_| AndroidInitError::AlreadyInitialized)?;
	// Only reachable by the single winner of the set above.
	let _ = ANDROID_CACHE_DIR.set(cache);
	Ok(())
}

pub fn init_android(files_dir: &str) -> Result<(), AndroidInitError> {
	let files = validate(files_dir)?;
	let cache = files.join("cache");
	store(files, cache)
}

pub fn init_android_with_cache(files_dir: &str, cache_dir: &str) -> Result<(), AndroidInitError> {
	let files = validate(files_dir)?;
	let cache = validate(cache_dir)?;
	store(files, cache)
}

// =============================================================================
// ndk-context auto-detection (feature-gated)
// =============================================================================

#[cfg(feature = "android-auto")]
mod auto {
	use super::{ANDROID_CACHE_DIR, ANDROID_FILES_DIR};
	use jni::objects::{JObject, JString};
	use std::path::PathBuf;

	/// Call `Context.<getter>().getAbsolutePath()` through JNI.
	fn context_path(getter: &str) -> Option<PathBuf> {
		let ctx = ndk_context::android_context();

		// Safety: ndk-context guarantees these pointers are valid once it has
		// been initialized; `from_raw` fails cleanly on a null VM before we
		// ever touch the context object.
		let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.ok()?;
		let mut env = vm.attach_current_thread().ok()?;
		let context = unsafe { JObject::from_raw(ctx.context().cast()) };
		if context.is_null() {
			return None;
		}

		let dir = env
			.call_method(&context, getter, "()Ljava/io/File;", &[])
			.ok()?
			.l()
			.ok()?;

		let abs_path: JString = env
			.call_method(&dir, "getAbsolutePath", "()Ljava/lang/String;", &[])
			.ok()?
			.l()
			.ok()?
			.into();

		let path_str: String = env.get_string(&abs_path).ok()?.into();
		Some(PathBuf::from(path_str))
	}

	/// Detect both dirs once and store them. Returns the files dir.
	///
	/// If detection fails (ndk-context not yet initialized) nothing is cached,
	/// so a later call can retry.
	pub(super) fn detect() -> Option<(PathBuf, PathBuf)> {
		let files = context_path("getFilesDir")?;
		let cache = context_path("getCacheDir")?;
		// A manual init may have raced us; whichever landed first wins and
		// both OnceLocks already agree, so ignoring the result is correct.
		let _ = ANDROID_FILES_DIR.set(files.clone());
		let _ = ANDROID_CACHE_DIR.set(cache.clone());
		Some((files, cache))
	}
}

// =============================================================================
// Helpers
// =============================================================================

/// Fallback when nothing has been initialized yet.
#[cfg(feature = "android-auto")]
fn detect() -> Option<(PathBuf, PathBuf)> {
	auto::detect()
}

#[cfg(not(feature = "android-auto"))]
fn detect() -> Option<(PathBuf, PathBuf)> {
	None
}

fn files_dir() -> Option<PathBuf> {
	match ANDROID_FILES_DIR.get() {
		Some(path) => Some(path.clone()),
		None => detect().map(|(files, _)| files),
	}
}

fn cache() -> Option<PathBuf> {
	match ANDROID_CACHE_DIR.get() {
		Some(path) => Some(path.clone()),
		None => detect().map(|(_, cache)| cache),
	}
}

// =============================================================================
// Directory implementations
// =============================================================================

pub fn home_dir() -> Option<PathBuf> {
	files_dir()
}

pub fn cache_dir() -> Option<PathBuf> {
	cache()
}

pub fn config_dir() -> Option<PathBuf> {
	files_dir()
}

pub fn config_local_dir() -> Option<PathBuf> {
	files_dir()
}

pub fn data_dir() -> Option<PathBuf> {
	files_dir()
}

pub fn data_local_dir() -> Option<PathBuf> {
	files_dir()
}

pub fn executable_dir() -> Option<PathBuf> {
	None
}

pub fn preference_dir() -> Option<PathBuf> {
	files_dir()
}

pub fn runtime_dir() -> Option<PathBuf> {
	None
}

pub fn state_dir() -> Option<PathBuf> {
	None
}

// Android apps don't have access to user directories from native code
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
	None
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
	files_dir().map(|f| f.join("tmp"))
}

pub fn library_dir() -> Option<PathBuf> {
	None
}

// =============================================================================
// Tests (pure validation only; OnceLock state is exercised in tests/android_init.rs)
// =============================================================================

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn validate_accepts_absolute() {
		assert_eq!(
			validate("/data/data/com.example/files"),
			Ok(PathBuf::from("/data/data/com.example/files"))
		);
	}

	#[test]
	fn validate_rejects_empty() {
		assert_eq!(validate(""), Err(AndroidInitError::InvalidPath));
	}

	#[test]
	fn validate_rejects_relative() {
		assert_eq!(validate("files"), Err(AndroidInitError::InvalidPath));
		assert_eq!(validate("./files"), Err(AndroidInitError::InvalidPath));
		assert_eq!(validate("../files"), Err(AndroidInitError::InvalidPath));
	}

	#[test]
	fn error_display_is_readable() {
		assert!(
			AndroidInitError::InvalidPath
				.to_string()
				.contains("absolute")
		);
		assert!(
			AndroidInitError::AlreadyInitialized
				.to_string()
				.contains("already")
		);
	}
}
