#![cfg_attr(docsrs, feature(doc_cfg))]
//! # sysdirs
//!
//! A low-level library with a minimal API that provides platform-specific, user-accessible
//! locations for finding and storing configuration, cache and other data on Linux,
//! Windows (≥ Vista), macOS, **iOS, Android, and WASM**.
//!
//! The library provides the location of these directories by leveraging the mechanisms defined by:
//!
//! * the [XDG base directory](https://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html)
//!   and the [XDG user directory](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/) specifications on Linux
//! * the [Known Folder](https://msdn.microsoft.com/en-us/library/windows/desktop/bb776911(v=vs.85).aspx) system on Windows
//! * the [Standard Directories](https://developer.apple.com/library/content/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemOverview/FileSystemOverview.html#//apple_ref/doc/uid/TP40010672-CH2-SW6) on macOS and iOS
//! * the app sandbox directories on Android (requires initialization)
//!
//! ## Usage
//!
//! ```rust
//! use sysdirs;
//!
//! sysdirs::home_dir();
//! // Lin: Some(/home/alice)
//! // Win: Some(C:\Users\Alice)
//! // Mac: Some(/Users/Alice)
//! // iOS: Some(/var/mobile/Containers/Data/Application/<UUID>)
//! // Android: Some(/data/data/com.example.app/files) [after init]
//!
//! sysdirs::cache_dir();
//! // Lin: Some(/home/alice/.cache)
//! // Win: Some(C:\Users\Alice\AppData\Local)
//! // Mac: Some(/Users/Alice/Library/Caches)
//! // iOS: Some(<sandbox>/Library/Caches)
//! // Android: Some(<filesDir>/cache)
//! ```
//!
//! ## Android Setup
//!
//! There are two ways to use sysdirs on Android:
//!
//! ### Option 1: Pure Rust Android apps (android-activity/ndk-glue)
//!
//! Enable the `android-auto` feature and paths are detected automatically:
//!
//! ```toml
//! [dependencies]
//! sysdirs = { version = "0.1", features = ["android-auto"] }
//! ```
//!
//! ### Option 2: Kotlin/Java apps embedding Rust
//!
//! Call `init_android()` once at startup from your JNI layer:
//!
//! ```rust,ignore
//! // Called from Kotlin/Java via JNI at app startup
//! sysdirs::init_android("/data/data/com.example.app/files");
//! ```
//!
//! The path should be obtained from `Context.getFilesDir()` in Kotlin/Java.
//!
//! ## Platform Support
//!
//! | Function | Linux | macOS | Windows | iOS | Android | WASM |
//! |----------|-------|-------|---------|-----|---------|------|
//! | `home_dir` | ✓ | ✓ | ✓ | ✓ | ✓* | ✗ |
//! | `cache_dir` | ✓ | ✓ | ✓ | ✓ | ✓* | ✗ |
//! | `config_dir` | ✓ | ✓ | ✓ | ✓ | ✓* | ✗ |
//! | `config_local_dir` | ✓ | ✓ | ✓ | ✓ | ✓* | ✗ |
//! | `data_dir` | ✓ | ✓ | ✓ | ✓ | ✓* | ✗ |
//! | `data_local_dir` | ✓ | ✓ | ✓ | ✓ | ✓* | ✗ |
//! | `document_dir` | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ |
//! | `download_dir` | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ |
//! | `preference_dir` | ✓ | ✓ | ✓ | ✓ | ✓* | ✗ |
//! | `audio_dir` | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ |
//! | `desktop_dir` | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ |
//! | `executable_dir` | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
//! | `font_dir` | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
//! | `picture_dir` | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ |
//! | `public_dir` | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ |
//! | `runtime_dir` | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
//! | `state_dir` | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
//! | `template_dir` | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ |
//! | `video_dir` | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ |
//!
//! \* Requires either the `android-auto` feature or [`init_android()`] to be called first

use std::io;
use std::path::Path;
use std::path::PathBuf;

// =============================================================================
// Path Extension Trait
// =============================================================================

/// Extension trait for `Option<PathBuf>` that adds chainable path operations.
///
/// This trait makes it easy to work with directory paths in a fluent style:
///
/// ```rust
/// use sysdirs::PathExt;
///
/// // Chain joins and ensure the directory exists
/// let app_cache = sysdirs::cache_dir()
///     .join("my-app")
///     .join("data")
///     .ensure();
/// ```
pub trait PathExt {
	/// Joins a path component to the contained path, if present.
	///
	/// This is chainable, allowing multiple joins in sequence.
	///
	/// # Example
	///
	/// ```rust
	/// use sysdirs::PathExt;
	///
	/// let path = sysdirs::data_dir()
	///     .join("my-app")
	///     .join("cache");
	/// // Linux: Some(/home/alice/.local/share/my-app/cache)
	/// ```
	fn join<P: AsRef<Path>>(self, path: P) -> Option<PathBuf>;

	/// Ensures the directory exists, creating it if necessary.
	///
	/// Returns the path if successful, or an error if:
	/// - The original `Option` was `None` (directory not available on this platform)
	/// - Directory creation failed (permissions, disk full, etc.)
	///
	/// # Example
	///
	/// ```rust,ignore
	/// use sysdirs::PathExt;
	///
	/// let app_data = sysdirs::data_dir()
	///     .join("my-app")
	///     .ensure()?;
	/// // Directory now exists, ready to use
	/// ```
	fn ensure(self) -> io::Result<PathBuf>;
}

impl PathExt for Option<PathBuf> {
	fn join<P: AsRef<Path>>(self, path: P) -> Option<PathBuf> {
		self.map(|p| p.join(path))
	}

	fn ensure(self) -> io::Result<PathBuf> {
		match self {
			Some(path) => {
				std::fs::create_dir_all(&path)?;
				Ok(path)
			}
			None => Err(io::Error::new(
				io::ErrorKind::NotFound,
				"directory not available on this platform",
			)),
		}
	}
}

// =============================================================================
// Platform Modules
// =============================================================================

#[cfg(any(
	target_os = "macos",
	target_os = "ios",
	target_os = "tvos",
	target_os = "watchos",
	target_os = "visionos"
))]
mod apple;
#[cfg(any(
	target_os = "macos",
	target_os = "ios",
	target_os = "tvos",
	target_os = "watchos",
	target_os = "visionos"
))]
use apple as platform;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as platform;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
use android as platform;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
use wasm as platform;

// Fallback for other platforms (FreeBSD, etc.)
#[cfg(not(any(
	target_os = "macos",
	target_os = "ios",
	target_os = "tvos",
	target_os = "watchos",
	target_os = "visionos",
	target_os = "linux",
	target_os = "windows",
	target_os = "android",
	target_arch = "wasm32"
)))]
mod unix;
#[cfg(not(any(
	target_os = "macos",
	target_os = "ios",
	target_os = "tvos",
	target_os = "watchos",
	target_os = "visionos",
	target_os = "linux",
	target_os = "windows",
	target_os = "android",
	target_arch = "wasm32"
)))]
use unix as platform;

// =============================================================================
// Apple Search Path Domain (Apple platforms only)
// =============================================================================

/// Search path domain for Apple platforms.
///
/// Controls which domain to search when looking up directories on macOS, iOS, etc.
/// Defaults to `User`.
///
/// This is only available on Apple platforms.
#[cfg(any(
	target_os = "macos",
	target_os = "ios",
	target_os = "tvos",
	target_os = "watchos",
	target_os = "visionos"
))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchPathDomain {
	/// User's home directory (e.g., ~/Library/...)
	#[default]
	User,
	/// Local machine (e.g., /Library/...)
	Local,
	/// Network locations (e.g., /Network/Library/...)
	Network,
	/// System (e.g., /System/Library/...)
	System,
}

/// Set the search path domain for Apple directory lookups.
///
/// By default, sysdirs uses the `User` domain which returns paths like `~/Library/Caches`.
/// System utilities or admin tools may want to use `Local` or `System` domains.
///
/// This function is only available on Apple platforms.
///
/// # Example
///
/// ```rust,ignore
/// use sysdirs::{SearchPathDomain, set_domain, cache_dir};
///
/// // Default: user domain
/// cache_dir(); // ~/Library/Caches
///
/// // Switch to local domain
/// set_domain(SearchPathDomain::Local);
/// cache_dir(); // /Library/Caches
/// ```
#[cfg(any(
	target_os = "macos",
	target_os = "ios",
	target_os = "tvos",
	target_os = "watchos",
	target_os = "visionos"
))]
pub fn set_domain(domain: SearchPathDomain) {
	platform::set_domain(domain);
}

// =============================================================================
// Android Initialization
// =============================================================================

/// Initialize Android-specific paths.
///
/// Must be called once at app startup on Android before using any directory functions.
/// The path should be obtained from `Context.getFilesDir()` in Kotlin/Java.
///
/// This function is only available on Android.
///
/// # Example
///
/// ```rust,ignore
/// // Called from JNI at app startup
/// sysdirs::init_android("/data/data/com.example.app/files");
/// ```
#[cfg(target_os = "android")]
pub fn init_android(files_dir: &str) {
	platform::init_android(files_dir);
}

/// Initialize Android-specific paths with separate directories.
///
/// Like [`init_android()`], but allows specifying both the files directory
/// and the cache directory separately. Use this if your app needs the actual
/// cache directory from `Context.getCacheDir()`.
///
/// This function is only available on Android.
///
/// # Example
///
/// ```rust,ignore
/// sysdirs::init_android_with_cache(
///     "/data/data/com.example.app/files",
///     "/data/data/com.example.app/cache"
/// );
/// ```
#[cfg(target_os = "android")]
pub fn init_android_with_cache(files_dir: &str, cache_dir: &str) {
	platform::init_android_with_cache(files_dir, cache_dir);
}

// =============================================================================
// Base Directories
// =============================================================================

/// Returns the path to the user's home directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                                    | Example                          |
/// | ------- | ---------------------------------------- | -------------------------------- |
/// | Linux   | `$HOME`                                  | /home/alice                      |
/// | macOS   | `$HOME`                                  | /Users/Alice                     |
/// | Windows | `{FOLDERID_Profile}`                     | C:\Users\Alice                   |
/// | iOS     | sandbox container                        | /var/mobile/.../&lt;UUID&gt;    |
/// | Android | files directory (after init)             | /data/data/com.example/files     |
/// | WASM    | `None`                                   |                                  |
pub fn home_dir() -> Option<PathBuf> {
	platform::home_dir()
}

/// Returns the path to the user's cache directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                               | Example                            |
/// | ------- | ----------------------------------- | ---------------------------------- |
/// | Linux   | `$XDG_CACHE_HOME` or `$HOME`/.cache | /home/alice/.cache                 |
/// | macOS   | `$HOME`/Library/Caches              | /Users/Alice/Library/Caches        |
/// | Windows | `{FOLDERID_LocalAppData}`           | C:\Users\Alice\AppData\Local       |
/// | iOS     | sandbox/Library/Caches              | &lt;sandbox&gt;/Library/Caches     |
/// | Android | files/cache (after init)            | /data/data/com.example/files/cache |
/// | WASM    | `None`                              |                                    |
pub fn cache_dir() -> Option<PathBuf> {
	platform::cache_dir()
}

/// Returns the path to the user's config directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                                 | Example                                    |
/// | ------- | ------------------------------------- | ------------------------------------------ |
/// | Linux   | `$XDG_CONFIG_HOME` or `$HOME`/.config | /home/alice/.config                        |
/// | macOS   | `$HOME`/Library/Application Support   | /Users/Alice/Library/Application Support   |
/// | Windows | `{FOLDERID_RoamingAppData}`           | C:\Users\Alice\AppData\Roaming             |
/// | iOS     | sandbox/Library/Application Support   | &lt;sandbox&gt;/Library/Application Support |
/// | Android | files directory (after init)          | /data/data/com.example/files               |
/// | WASM    | `None`                                |                                            |
pub fn config_dir() -> Option<PathBuf> {
	platform::config_dir()
}

/// Returns the path to the user's local config directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                                 | Example                                    |
/// | ------- | ------------------------------------- | ------------------------------------------ |
/// | Linux   | `$XDG_CONFIG_HOME` or `$HOME`/.config | /home/alice/.config                        |
/// | macOS   | `$HOME`/Library/Application Support   | /Users/Alice/Library/Application Support   |
/// | Windows | `{FOLDERID_LocalAppData}`             | C:\Users\Alice\AppData\Local               |
/// | iOS     | sandbox/Library/Application Support   | &lt;sandbox&gt;/Library/Application Support |
/// | Android | files directory (after init)          | /data/data/com.example/files               |
/// | WASM    | `None`                                |                                            |
pub fn config_local_dir() -> Option<PathBuf> {
	platform::config_local_dir()
}

/// Returns the path to the user's data directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                                       | Example                                    |
/// | ------- | ------------------------------------------- | ------------------------------------------ |
/// | Linux   | `$XDG_DATA_HOME` or `$HOME`/.local/share    | /home/alice/.local/share                   |
/// | macOS   | `$HOME`/Library/Application Support         | /Users/Alice/Library/Application Support   |
/// | Windows | `{FOLDERID_RoamingAppData}`                 | C:\Users\Alice\AppData\Roaming             |
/// | iOS     | sandbox/Library/Application Support         | &lt;sandbox&gt;/Library/Application Support |
/// | Android | files directory (after init)                | /data/data/com.example/files               |
/// | WASM    | `None`                                      |                                            |
pub fn data_dir() -> Option<PathBuf> {
	platform::data_dir()
}

/// Returns the path to the user's local data directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                                       | Example                                    |
/// | ------- | ------------------------------------------- | ------------------------------------------ |
/// | Linux   | `$XDG_DATA_HOME` or `$HOME`/.local/share    | /home/alice/.local/share                   |
/// | macOS   | `$HOME`/Library/Application Support         | /Users/Alice/Library/Application Support   |
/// | Windows | `{FOLDERID_LocalAppData}`                   | C:\Users\Alice\AppData\Local               |
/// | iOS     | sandbox/Library/Application Support         | &lt;sandbox&gt;/Library/Application Support |
/// | Android | files directory (after init)                | /data/data/com.example/files               |
/// | WASM    | `None`                                      |                                            |
pub fn data_local_dir() -> Option<PathBuf> {
	platform::data_local_dir()
}

/// Returns the path to the user's executable directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                                    | Example                    |
/// | ------- | ---------------------------------------- | -------------------------- |
/// | Linux   | `$XDG_BIN_HOME` or `$HOME`/.local/bin    | /home/alice/.local/bin     |
/// | macOS   | `None`                                   |                            |
/// | Windows | `None`                                   |                            |
/// | iOS     | `None`                                   |                            |
/// | Android | `None`                                   |                            |
/// | WASM    | `None`                                   |                            |
pub fn executable_dir() -> Option<PathBuf> {
	platform::executable_dir()
}

/// Returns the path to the user's preference directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                                 | Example                                  |
/// | ------- | ------------------------------------- | ---------------------------------------- |
/// | Linux   | `$XDG_CONFIG_HOME` or `$HOME`/.config | /home/alice/.config                      |
/// | macOS   | `$HOME`/Library/Preferences           | /Users/Alice/Library/Preferences         |
/// | Windows | `{FOLDERID_RoamingAppData}`           | C:\Users\Alice\AppData\Roaming           |
/// | iOS     | sandbox/Library/Preferences           | &lt;sandbox&gt;/Library/Preferences      |
/// | Android | files directory (after init)          | /data/data/com.example/files             |
/// | WASM    | `None`                                |                                          |
pub fn preference_dir() -> Option<PathBuf> {
	platform::preference_dir()
}

/// Returns the path to the user's runtime directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value              | Example         |
/// | ------- | ------------------ | --------------- |
/// | Linux   | `$XDG_RUNTIME_DIR` | /run/user/1000  |
/// | macOS   | `None`             |                 |
/// | Windows | `None`             |                 |
/// | iOS     | `None`             |                 |
/// | Android | `None`             |                 |
/// | WASM    | `None`             |                 |
pub fn runtime_dir() -> Option<PathBuf> {
	platform::runtime_dir()
}

/// Returns the path to the user's state directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                                       | Example                    |
/// | ------- | ------------------------------------------- | -------------------------- |
/// | Linux   | `$XDG_STATE_HOME` or `$HOME`/.local/state   | /home/alice/.local/state   |
/// | macOS   | `None`                                      |                            |
/// | Windows | `None`                                      |                            |
/// | iOS     | `None`                                      |                            |
/// | Android | `None`                                      |                            |
/// | WASM    | `None`                                      |                            |
pub fn state_dir() -> Option<PathBuf> {
	platform::state_dir()
}

// =============================================================================
// User Directories
// =============================================================================

/// Returns the path to the user's audio directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                 | Example                  |
/// | ------- | --------------------- | ------------------------ |
/// | Linux   | `XDG_MUSIC_DIR`       | /home/alice/Music        |
/// | macOS   | `$HOME`/Music         | /Users/Alice/Music       |
/// | Windows | `{FOLDERID_Music}`    | C:\Users\Alice\Music     |
/// | iOS     | `None`                |                          |
/// | Android | `None`                |                          |
/// | WASM    | `None`                |                          |
pub fn audio_dir() -> Option<PathBuf> {
	platform::audio_dir()
}

/// Returns the path to the user's desktop directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                 | Example                  |
/// | ------- | --------------------- | ------------------------ |
/// | Linux   | `XDG_DESKTOP_DIR`     | /home/alice/Desktop      |
/// | macOS   | `$HOME`/Desktop       | /Users/Alice/Desktop     |
/// | Windows | `{FOLDERID_Desktop}`  | C:\Users\Alice\Desktop   |
/// | iOS     | `None`                |                          |
/// | Android | `None`                |                          |
/// | WASM    | `None`                |                          |
pub fn desktop_dir() -> Option<PathBuf> {
	platform::desktop_dir()
}

/// Returns the path to the user's document directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                   | Example                    |
/// | ------- | ----------------------- | -------------------------- |
/// | Linux   | `XDG_DOCUMENTS_DIR`     | /home/alice/Documents      |
/// | macOS   | `$HOME`/Documents       | /Users/Alice/Documents     |
/// | Windows | `{FOLDERID_Documents}`  | C:\Users\Alice\Documents   |
/// | iOS     | sandbox/Documents       | &lt;sandbox&gt;/Documents  |
/// | Android | `None`                  |                            |
/// | WASM    | `None`                  |                            |
pub fn document_dir() -> Option<PathBuf> {
	platform::document_dir()
}

/// Returns the path to the user's download directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                   | Example                    |
/// | ------- | ----------------------- | -------------------------- |
/// | Linux   | `XDG_DOWNLOAD_DIR`      | /home/alice/Downloads      |
/// | macOS   | `$HOME`/Downloads       | /Users/Alice/Downloads     |
/// | Windows | `{FOLDERID_Downloads}`  | C:\Users\Alice\Downloads   |
/// | iOS     | `None`                  |                            |
/// | Android | `None`                  |                            |
/// | WASM    | `None`                  |                            |
pub fn download_dir() -> Option<PathBuf> {
	platform::download_dir()
}

/// Returns the path to the user's font directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                                              | Example                            |
/// | ------- | -------------------------------------------------- | ---------------------------------- |
/// | Linux   | `$XDG_DATA_HOME`/fonts or `$HOME`/.local/share/fonts | /home/alice/.local/share/fonts   |
/// | macOS   | `$HOME`/Library/Fonts                              | /Users/Alice/Library/Fonts         |
/// | Windows | `None`                                             |                                    |
/// | iOS     | `None`                                             |                                    |
/// | Android | `None`                                             |                                    |
/// | WASM    | `None`                                             |                                    |
pub fn font_dir() -> Option<PathBuf> {
	platform::font_dir()
}

/// Returns the path to the user's picture directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                   | Example                    |
/// | ------- | ----------------------- | -------------------------- |
/// | Linux   | `XDG_PICTURES_DIR`      | /home/alice/Pictures       |
/// | macOS   | `$HOME`/Pictures        | /Users/Alice/Pictures      |
/// | Windows | `{FOLDERID_Pictures}`   | C:\Users\Alice\Pictures    |
/// | iOS     | `None`                  |                            |
/// | Android | `None`                  |                            |
/// | WASM    | `None`                  |                            |
pub fn picture_dir() -> Option<PathBuf> {
	platform::picture_dir()
}

/// Returns the path to the user's public directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                   | Example                    |
/// | ------- | ----------------------- | -------------------------- |
/// | Linux   | `XDG_PUBLICSHARE_DIR`   | /home/alice/Public         |
/// | macOS   | `$HOME`/Public          | /Users/Alice/Public        |
/// | Windows | `{FOLDERID_Public}`     | C:\Users\Public            |
/// | iOS     | `None`                  |                            |
/// | Android | `None`                  |                            |
/// | WASM    | `None`                  |                            |
pub fn public_dir() -> Option<PathBuf> {
	platform::public_dir()
}

/// Returns the path to the user's template directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                   | Example                              |
/// | ------- | ----------------------- | ------------------------------------ |
/// | Linux   | `XDG_TEMPLATES_DIR`     | /home/alice/Templates                |
/// | macOS   | `None`                  |                                      |
/// | Windows | `{FOLDERID_Templates}`  | C:\Users\Alice\AppData\Roaming\Microsoft\Windows\Templates |
/// | iOS     | `None`                  |                                      |
/// | Android | `None`                  |                                      |
/// | WASM    | `None`                  |                                      |
pub fn template_dir() -> Option<PathBuf> {
	platform::template_dir()
}

/// Returns the path to the user's video directory.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                 | Example                  |
/// | ------- | --------------------- | ------------------------ |
/// | Linux   | `XDG_VIDEOS_DIR`      | /home/alice/Videos       |
/// | macOS   | `$HOME`/Movies        | /Users/Alice/Movies      |
/// | Windows | `{FOLDERID_Videos}`   | C:\Users\Alice\Videos    |
/// | iOS     | `None`                |                          |
/// | Android | `None`                |                          |
/// | WASM    | `None`                |                          |
pub fn video_dir() -> Option<PathBuf> {
	platform::video_dir()
}

// =============================================================================
// sysdirs Extensions
// =============================================================================

/// Returns the path to the app's temporary directory.
///
/// This is a sysdirs extension not present in the `dirs` crate.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                        | Example                      |
/// | ------- | ---------------------------- | ---------------------------- |
/// | Linux   | `$TMPDIR` or /tmp            | /tmp                         |
/// | macOS   | `$TMPDIR`                    | /var/folders/.../T/          |
/// | Windows | `{FOLDERID_LocalAppData}`\Temp | C:\Users\Alice\AppData\Local\Temp |
/// | iOS     | sandbox/tmp                  | &lt;sandbox&gt;/tmp          |
/// | Android | files/tmp (after init)       | /data/data/com.example/files/tmp |
/// | WASM    | `None`                       |                              |
pub fn temp_dir() -> Option<PathBuf> {
	platform::temp_dir()
}

/// Returns the path to the app's Library directory (Apple platforms only).
///
/// This is a sysdirs extension not present in the `dirs` crate.
///
/// The returned value depends on the operating system and is either a `Some`, containing a value
/// from the following table, or a `None`.
///
/// |Platform | Value                        | Example                      |
/// | ------- | ---------------------------- | ---------------------------- |
/// | Linux   | `None`                       |                              |
/// | macOS   | `$HOME`/Library              | /Users/Alice/Library         |
/// | Windows | `None`                       |                              |
/// | iOS     | sandbox/Library              | &lt;sandbox&gt;/Library      |
/// | Android | `None`                       |                              |
/// | WASM    | `None`                       |                              |
pub fn library_dir() -> Option<PathBuf> {
	platform::library_dir()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_home_dir() {
		// On most platforms we should get something
		#[cfg(not(target_arch = "wasm32"))]
		assert!(home_dir().is_some());
	}

	#[test]
	fn test_cache_dir() {
		#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
		assert!(cache_dir().is_some());
	}

	#[test]
	fn test_config_dir() {
		#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
		assert!(config_dir().is_some());
	}

	#[test]
	fn test_data_dir() {
		#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
		assert!(data_dir().is_some());
	}

	#[test]
	#[cfg(target_os = "android")]
	fn test_android_init() {
		init_android("/data/data/com.test/files");
		assert_eq!(home_dir(), Some(PathBuf::from("/data/data/com.test/files")));
		assert_eq!(
			cache_dir(),
			Some(PathBuf::from("/data/data/com.test/files/cache"))
		);
	}
}
