//! Apple platform implementation (macOS, iOS, tvOS, watchOS, visionOS)
//!
//! Uses the sysdir FFI for proper sandbox-aware directory lookups.

use crate::SearchPathDomain;
use std::ffi::{CStr, c_char};
use std::path::{Path, PathBuf};
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
	fn sysdir_start_search_path_enumeration(dir: u32, domain_mask: u32) -> SysdirState;

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
	let raw = first_sysdir_entry(dir as u32, get_domain_mask())?;
	let home = std::env::var_os("HOME").map(PathBuf::from);
	expand_home(&raw, home.as_deref())
}

/// Ask sysdir for the first entry of `dir` in `domain_mask`, as the raw
/// string it returns (possibly `~/...`). `None` if sysdir has nothing.
///
/// Takes the raw id rather than [`SysdirDirectory`] so tests can hand it an
/// id sysdir rejects and exercise the failure path against the real libc.
fn first_sysdir_entry(dir: u32, domain_mask: u32) -> Option<String> {
	let mut buf = [0 as c_char; PATH_MAX];

	// Safety: `buf` is PATH_MAX bytes, which is what sysdir requires, and it
	// NUL-terminates whatever it writes. A finished state with an empty
	// buffer means no entry; a finished state with a non-empty buffer means
	// this was the last (and only) entry, which is still a valid result.
	unsafe {
		let state = sysdir_start_search_path_enumeration(dir, domain_mask);
		if state.is_finished() {
			return None;
		}
		let state = sysdir_get_next_search_path_enumeration(state, buf.as_mut_ptr());
		if state.is_finished() && buf[0] == 0 {
			return None;
		}
		CStr::from_ptr(buf.as_ptr())
			.to_str()
			.ok()
			.map(str::to_owned)
	}
}

/// Expand sysdir's `~` shorthand (user domain) against `home`.
fn expand_home(raw: &str, home: Option<&Path>) -> Option<PathBuf> {
	if let Some(rest) = raw.strip_prefix("~/") {
		home.map(|h| h.join(rest))
	} else if raw == "~" {
		home.map(Path::to_path_buf)
	} else {
		Some(PathBuf::from(raw))
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

/// `confstr(_CS_DARWIN_USER_TEMP_DIR)`, or `None` if the call fails.
fn darwin_user_temp_dir() -> Option<PathBuf> {
	confstr_path(CS_DARWIN_USER_TEMP_DIR)
}

/// One `confstr` lookup as an absolute path, or `None` if the call fails,
/// the value doesn't fit `PATH_MAX`, or it isn't absolute.
///
/// Takes the raw name so tests can pass one confstr rejects.
fn confstr_path(name: std::ffi::c_int) -> Option<PathBuf> {
	let mut buf = [0 as c_char; PATH_MAX];
	// Safety: buf is a valid, writable, PATH_MAX-byte buffer and confstr
	// NUL-terminates within `len` on success.
	let needed = unsafe { confstr(name, buf.as_mut_ptr(), PATH_MAX) };
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
	use super::*;

	// -------------------------------------------------------------------------
	// expand_home (pure)
	// -------------------------------------------------------------------------

	#[test]
	fn expand_home_tilde_slash() {
		let home = Path::new("/Users/test");
		assert_eq!(
			expand_home("~/Library/Caches", Some(home)),
			Some(PathBuf::from("/Users/test/Library/Caches"))
		);
	}

	#[test]
	fn expand_home_bare_tilde() {
		let home = Path::new("/Users/test");
		assert_eq!(
			expand_home("~", Some(home)),
			Some(PathBuf::from("/Users/test"))
		);
	}

	#[test]
	fn expand_home_absolute_passthrough() {
		let home = Path::new("/Users/test");
		assert_eq!(
			expand_home("/Library", Some(home)),
			Some(PathBuf::from("/Library"))
		);
		assert_eq!(
			expand_home("/Library", None),
			Some(PathBuf::from("/Library"))
		);
	}

	#[test]
	fn expand_home_needs_home_for_tilde() {
		assert_eq!(expand_home("~/x", None), None);
		assert_eq!(expand_home("~", None), None);
	}

	// -------------------------------------------------------------------------
	// first_sysdir_entry (against the real libc)
	// -------------------------------------------------------------------------

	#[test]
	fn sysdir_known_dir_returns_entry() {
		let raw = first_sysdir_entry(SysdirDirectory::Library as u32, SYSDIR_DOMAIN_MASK_USER)
			.expect("Library in user domain");
		assert_eq!(raw, "~/Library");
	}

	#[test]
	fn sysdir_local_domain_is_absolute() {
		assert_eq!(
			first_sysdir_entry(SysdirDirectory::Caches as u32, SYSDIR_DOMAIN_MASK_LOCAL),
			Some("/Library/Caches".to_owned())
		);
	}

	#[test]
	fn sysdir_invalid_dir_is_none() {
		// 0 is not a valid sysdir_search_path_directory_t; enumeration never starts.
		assert_eq!(first_sysdir_entry(0, SYSDIR_DOMAIN_MASK_USER), None);
		assert_eq!(first_sysdir_entry(u32::MAX, SYSDIR_DOMAIN_MASK_USER), None);
	}

	#[test]
	fn sysdir_started_but_empty_is_none() {
		// Documents exists as a directory type but has no entry outside the
		// user domain: enumeration starts, then yields nothing.
		assert_eq!(
			first_sysdir_entry(SysdirDirectory::Document as u32, SYSDIR_DOMAIN_MASK_LOCAL),
			None
		);
	}

	#[test]
	fn sysdir_empty_domain_mask_is_none() {
		// No domains selected: nothing to enumerate.
		assert_eq!(first_sysdir_entry(SysdirDirectory::Library as u32, 0), None);
	}

	#[test]
	fn domain_mask_round_trips() {
		for d in [
			SearchPathDomain::User,
			SearchPathDomain::Local,
			SearchPathDomain::Network,
			SearchPathDomain::System,
		] {
			assert_eq!(domain_from_mask(domain_mask(d)), d);
		}
		// Unknown masks fall back to User rather than panicking.
		assert_eq!(domain_from_mask(0), SearchPathDomain::User);
		assert_eq!(domain_from_mask(0xFF), SearchPathDomain::User);
	}

	// -------------------------------------------------------------------------
	// confstr (against the real libc)
	// -------------------------------------------------------------------------

	#[test]
	fn confstr_user_temp_dir_is_absolute_and_exists() {
		let p = confstr_path(CS_DARWIN_USER_TEMP_DIR).expect("confstr temp dir");
		assert!(p.is_absolute());
		assert!(p.exists());
	}

	#[test]
	fn confstr_invalid_name_is_none() {
		assert_eq!(confstr_path(-1), None);
	}
}
