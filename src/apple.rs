//! Apple platform implementation (macOS, iOS, tvOS, watchOS, visionOS)
//!
//! Uses the sysdir FFI for proper sandbox-aware directory lookups.

use crate::SearchPathDomain;
use std::cell::Cell;
use std::ffi::CStr;
use std::path::PathBuf;

const PATH_MAX: usize = 1024;

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
enum SysdirDirectory {
	Library = 5,
	Document = 9,
	Desktop = 12,
	Caches = 13,
	ApplicationSupport = 14,
	Downloads = 15,
	Movies = 17,
	Music = 18,
	Pictures = 19,
	SharedPublic = 21,
}

const SYSDIR_DOMAIN_MASK_USER: u32 = 1;
const SYSDIR_DOMAIN_MASK_LOCAL: u32 = 2;
const SYSDIR_DOMAIN_MASK_NETWORK: u32 = 4;
const SYSDIR_DOMAIN_MASK_SYSTEM: u32 = 8;

#[repr(transparent)]
struct SysdirState(u32);

impl SysdirState {
	fn is_finished(&self) -> bool {
		self.0 == 0
	}
}

unsafe extern "C" {
	fn sysdir_start_search_path_enumeration(
		dir: SysdirDirectory,
		domain_mask: u32,
	) -> SysdirState;

	fn sysdir_get_next_search_path_enumeration(
		state: SysdirState,
		path: *mut std::ffi::c_char,
	) -> SysdirState;
}

// =============================================================================
// Domain management (thread-local for test isolation)
// =============================================================================

thread_local! {
	static CURRENT_DOMAIN: Cell<u32> = const { Cell::new(SYSDIR_DOMAIN_MASK_USER) };
}

pub fn set_domain(domain: SearchPathDomain) {
	let mask = match domain {
		SearchPathDomain::User => SYSDIR_DOMAIN_MASK_USER,
		SearchPathDomain::Local => SYSDIR_DOMAIN_MASK_LOCAL,
		SearchPathDomain::Network => SYSDIR_DOMAIN_MASK_NETWORK,
		SearchPathDomain::System => SYSDIR_DOMAIN_MASK_SYSTEM,
	};
	CURRENT_DOMAIN.set(mask);
}

fn get_domain_mask() -> u32 {
	CURRENT_DOMAIN.get()
}

// =============================================================================
// Core sysdir lookup
// =============================================================================

/// Get the first path for a directory type in the current domain.
fn sysdir_path(dir: SysdirDirectory) -> Option<PathBuf> {
	let mut path_buf = [0i8; PATH_MAX];
	let domain = get_domain_mask();

	unsafe {
		let state = sysdir_start_search_path_enumeration(dir, domain);
		if state.is_finished() {
			return None;
		}

		let state = sysdir_get_next_search_path_enumeration(state, path_buf.as_mut_ptr());
		if state.is_finished() && path_buf[0] == 0 {
			return None;
		}

		let c_str = CStr::from_ptr(path_buf.as_ptr());
		let path_str = c_str.to_str().ok()?;

		// Handle ~ expansion for user domain
		if path_str.starts_with("~/") {
			let home = std::env::var_os("HOME")?;
			Some(PathBuf::from(home).join(&path_str[2..]))
		} else if path_str == "~" {
			std::env::var_os("HOME").map(PathBuf::from)
		} else {
			Some(PathBuf::from(path_str))
		}
	}
}

// =============================================================================
// Directory implementations
// =============================================================================

pub fn home_dir() -> Option<PathBuf> {
	// sysdir doesn't have a "home" directory type, use $HOME
	std::env::var_os("HOME").map(PathBuf::from)
}

pub fn cache_dir() -> Option<PathBuf> {
	sysdir_path(SysdirDirectory::Caches)
}

pub fn config_dir() -> Option<PathBuf> {
	sysdir_path(SysdirDirectory::ApplicationSupport)
}

pub fn config_local_dir() -> Option<PathBuf> {
	config_dir()
}

pub fn data_dir() -> Option<PathBuf> {
	sysdir_path(SysdirDirectory::ApplicationSupport)
}

pub fn data_local_dir() -> Option<PathBuf> {
	data_dir()
}

pub fn executable_dir() -> Option<PathBuf> {
	None
}

pub fn preference_dir() -> Option<PathBuf> {
	// sysdir doesn't have Preferences, derive from Library
	library_dir().map(|l| l.join("Preferences"))
}

pub fn runtime_dir() -> Option<PathBuf> {
	None
}

pub fn state_dir() -> Option<PathBuf> {
	None
}

// User directories - only available on macOS, not iOS/tvOS/etc
#[cfg(target_os = "macos")]
pub fn audio_dir() -> Option<PathBuf> {
	sysdir_path(SysdirDirectory::Music)
}

#[cfg(not(target_os = "macos"))]
pub fn audio_dir() -> Option<PathBuf> {
	None
}

#[cfg(target_os = "macos")]
pub fn desktop_dir() -> Option<PathBuf> {
	sysdir_path(SysdirDirectory::Desktop)
}

#[cfg(not(target_os = "macos"))]
pub fn desktop_dir() -> Option<PathBuf> {
	None
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn document_dir() -> Option<PathBuf> {
	sysdir_path(SysdirDirectory::Document)
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
pub fn document_dir() -> Option<PathBuf> {
	None
}

#[cfg(target_os = "macos")]
pub fn download_dir() -> Option<PathBuf> {
	sysdir_path(SysdirDirectory::Downloads)
}

#[cfg(not(target_os = "macos"))]
pub fn download_dir() -> Option<PathBuf> {
	None
}

#[cfg(target_os = "macos")]
pub fn font_dir() -> Option<PathBuf> {
	// sysdir doesn't have Fonts, derive from Library
	library_dir().map(|l| l.join("Fonts"))
}

#[cfg(not(target_os = "macos"))]
pub fn font_dir() -> Option<PathBuf> {
	None
}

#[cfg(target_os = "macos")]
pub fn picture_dir() -> Option<PathBuf> {
	sysdir_path(SysdirDirectory::Pictures)
}

#[cfg(not(target_os = "macos"))]
pub fn picture_dir() -> Option<PathBuf> {
	None
}

#[cfg(target_os = "macos")]
pub fn public_dir() -> Option<PathBuf> {
	sysdir_path(SysdirDirectory::SharedPublic)
}

#[cfg(not(target_os = "macos"))]
pub fn public_dir() -> Option<PathBuf> {
	None
}

pub fn template_dir() -> Option<PathBuf> {
	None
}

#[cfg(target_os = "macos")]
pub fn video_dir() -> Option<PathBuf> {
	sysdir_path(SysdirDirectory::Movies)
}

#[cfg(not(target_os = "macos"))]
pub fn video_dir() -> Option<PathBuf> {
	None
}

// =============================================================================
// sysdirs extensions
// =============================================================================

pub fn temp_dir() -> Option<PathBuf> {
	std::env::var_os("TMPDIR").map(PathBuf::from)
}

pub fn library_dir() -> Option<PathBuf> {
	sysdir_path(SysdirDirectory::Library)
}
