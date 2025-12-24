//! Android platform implementation
//!
//! Android apps run in a sandbox with randomized paths. This module supports two
//! initialization methods:
//!
//! 1. Manual init via `init_android()` - for Kotlin/Java apps embedding Rust
//! 2. Auto-detection via ndk-context - for pure Rust Android apps (requires `android-auto` feature)

use std::path::PathBuf;
use std::sync::OnceLock;

// =============================================================================
// Initialization
// =============================================================================

static ANDROID_FILES_DIR: OnceLock<PathBuf> = OnceLock::new();
static ANDROID_CACHE_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn init_android(files_dir: &str) {
	let path = PathBuf::from(files_dir);
	let _ = ANDROID_FILES_DIR.set(path.clone());
	let _ = ANDROID_CACHE_DIR.set(path.join("cache"));
}

pub fn init_android_with_cache(files_dir: &str, cache_dir: &str) {
	let _ = ANDROID_FILES_DIR.set(PathBuf::from(files_dir));
	let _ = ANDROID_CACHE_DIR.set(PathBuf::from(cache_dir));
}

// =============================================================================
// ndk-context auto-detection (feature-gated)
// =============================================================================

/// Try to get files dir from ndk-context (for pure Rust Android apps).
/// Returns None if ndk-context isn't initialized or feature isn't enabled.
#[cfg(feature = "android-auto")]
fn try_ndk_context_files_dir() -> Option<PathBuf> {
	use jni::objects::{JObject, JString};

	// Get the Android context from ndk-context
	let ctx = ndk_context::android_context();

	// Safety: ndk-context guarantees these pointers are valid when initialized
	let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.ok()?;
	let mut env = vm.attach_current_thread().ok()?;
	let context = unsafe { JObject::from_raw(ctx.context().cast()) };

	// Call context.getFilesDir().getAbsolutePath()
	let files_dir = env
		.call_method(&context, "getFilesDir", "()Ljava/io/File;", &[])
		.ok()?
		.l()
		.ok()?;

	let abs_path: JString = env
		.call_method(&files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])
		.ok()?
		.l()
		.ok()?
		.into();

	let path_str: String = env.get_string(&abs_path).ok()?.into();
	Some(PathBuf::from(path_str))
}

/// Try to get cache dir from ndk-context (for pure Rust Android apps).
#[cfg(feature = "android-auto")]
fn try_ndk_context_cache_dir() -> Option<PathBuf> {
	use jni::objects::{JObject, JString};

	let ctx = ndk_context::android_context();
	let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.ok()?;
	let mut env = vm.attach_current_thread().ok()?;
	let context = unsafe { JObject::from_raw(ctx.context().cast()) };

	let cache_dir = env
		.call_method(&context, "getCacheDir", "()Ljava/io/File;", &[])
		.ok()?
		.l()
		.ok()?;

	let abs_path: JString = env
		.call_method(&cache_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])
		.ok()?
		.l()
		.ok()?
		.into();

	let path_str: String = env.get_string(&abs_path).ok()?.into();
	Some(PathBuf::from(path_str))
}

// =============================================================================
// Helpers
// =============================================================================

fn files_dir() -> Option<PathBuf> {
	// First check manual init
	if let Some(path) = ANDROID_FILES_DIR.get() {
		return Some(path.clone());
	}

	// Then try ndk-context if feature is enabled
	#[cfg(feature = "android-auto")]
	{
		return try_ndk_context_files_dir();
	}

	#[cfg(not(feature = "android-auto"))]
	None
}

fn cache() -> Option<PathBuf> {
	// First check manual init
	if let Some(path) = ANDROID_CACHE_DIR.get() {
		return Some(path.clone());
	}

	// Then try ndk-context if feature is enabled
	#[cfg(feature = "android-auto")]
	{
		return try_ndk_context_cache_dir();
	}

	#[cfg(not(feature = "android-auto"))]
	None
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
