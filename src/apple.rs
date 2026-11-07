//! Apple platform implementation (macOS, iOS, tvOS, watchOS, visionOS)
//!
//! Uses the sysdir FFI for proper sandbox-aware directory lookups.

use crate::SearchPathDomain;
use std::ffi::{CStr, c_char};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

const PATH_MAX: usize = 1024;

// Only macOS exposes the user-media directories; on iOS and friends those
// variants are declared for parity with the C enum but never constructed.
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
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
	fn sysdir_start_search_path_enumeration(dir: SysdirDirectory, domain_mask: u32) -> SysdirState;

	fn sysdir_get_next_search_path_enumeration(
		state: SysdirState,
		path: *mut c_char,
	) -> SysdirState;
}

// =============================================================================
// Domain management (process-wide)
// =============================================================================

/// The domain mask every lookup uses. Process-wide: a domain set on one
/// thread applies to lookups on every thread.
static CURRENT_DOMAIN: AtomicU32 = AtomicU32::new(SYSDIR_DOMAIN_MASK_USER);

fn domain_mask(domain: SearchPathDomain) -> u32 {
	match domain {
		SearchPathDomain::User => SYSDIR_DOMAIN_MASK_USER,
		SearchPathDomain::Local => SYSDIR_DOMAIN_MASK_LOCAL,
		SearchPathDomain::Network => SYSDIR_DOMAIN_MASK_NETWORK,
		SearchPathDomain::System => SYSDIR_DOMAIN_MASK_SYSTEM,
	}
}

fn domain_from_mask(mask: u32) -> SearchPathDomain {
	match mask {
		SYSDIR_DOMAIN_MASK_LOCAL => SearchPathDomain::Local,
		SYSDIR_DOMAIN_MASK_NETWORK => SearchPathDomain::Network,
		SYSDIR_DOMAIN_MASK_SYSTEM => SearchPathDomain::System,
		_ => SearchPathDomain::User,
	}
}

pub fn set_domain(domain: SearchPathDomain) {
	CURRENT_DOMAIN.store(domain_mask(domain), Ordering::Relaxed);
}

pub fn domain() -> SearchPathDomain {
	domain_from_mask(get_domain_mask())
}

fn get_domain_mask() -> u32 {
	CURRENT_DOMAIN.load(Ordering::Relaxed)
}

// =============================================================================
// Core sysdir lookup
// =============================================================================

/// Get the first path for a directory type in the current domain.
fn sysdir_path(dir: SysdirDirectory) -> Option<PathBuf> {
	let mut path_buf = [0 as c_char; PATH_MAX];
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
		if let Some(rest) = path_str.strip_prefix("~/") {
			let home = std::env::var_os("HOME")?;
			Some(PathBuf::from(home).join(rest))
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

/// The per-user temp directory.
///
/// `$TMPDIR` is honored when set to an absolute path. When it is absent, which
/// is normal for launchd agents, XPC services and sandboxed helpers, we ask the
/// system directly via `confstr(_CS_DARWIN_USER_TEMP_DIR)`, which is where the
/// login shell gets `TMPDIR` from in the first place. `/tmp` is the last resort.
pub fn temp_dir() -> Option<PathBuf> {
	std::env::var_os("TMPDIR")
		.map(PathBuf::from)
		.filter(|p| p.is_absolute())
		.or_else(darwin_user_temp_dir)
		.or_else(|| Some(PathBuf::from("/tmp")))
}

const CS_DARWIN_USER_TEMP_DIR: std::ffi::c_int = 65537;

unsafe extern "C" {
	fn confstr(name: std::ffi::c_int, buf: *mut c_char, len: usize) -> usize;
}

/// `confstr(_CS_DARWIN_USER_TEMP_DIR)`, or `None` if the call fails or
/// returns a value that doesn't fit `PATH_MAX`.
fn darwin_user_temp_dir() -> Option<PathBuf> {
	let mut buf = [0 as c_char; PATH_MAX];
	// Safety: buf is a valid, writable, PATH_MAX-byte buffer and confstr
	// NUL-terminates within `len` on success.
	let needed = unsafe { confstr(CS_DARWIN_USER_TEMP_DIR, buf.as_mut_ptr(), PATH_MAX) };
	if needed == 0 || needed > PATH_MAX {
		return None;
	}
	let s = unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().ok()?;
	let path = PathBuf::from(s);
	path.is_absolute().then_some(path)
}

pub fn library_dir() -> Option<PathBuf> {
	sysdir_path(SysdirDirectory::Library)
}
