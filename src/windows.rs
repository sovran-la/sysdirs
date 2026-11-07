//! Windows platform implementation
//!
//! Uses the Known Folders API (`SHGetKnownFolderPath`) so that redirected
//! folders (OneDrive "Known Folder Move", group-policy redirection, custom
//! `Documents` locations) resolve to where the user's files actually are,
//! rather than guessing from `USERPROFILE`.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::UI::Shell::{
	FOLDERID_Desktop, FOLDERID_Documents, FOLDERID_Downloads, FOLDERID_LocalAppData,
	FOLDERID_Music, FOLDERID_Pictures, FOLDERID_Profile, FOLDERID_Public, FOLDERID_RoamingAppData,
	FOLDERID_Templates, FOLDERID_Videos, KF_FLAG_DEFAULT, SHGetKnownFolderPath,
};
use windows_sys::core::GUID;

// =============================================================================
// Known Folders
// =============================================================================

/// Resolve a known folder for the current user.
///
/// Returns `None` if the shell reports an error (folder not defined for this
/// user, or a redirected location that doesn't exist with `KF_FLAG_DEFAULT`).
fn known_folder(id: &GUID) -> Option<PathBuf> {
	let mut raw: *mut u16 = std::ptr::null_mut();

	// Safety: `id` is a valid GUID reference, the token is null (current
	// user), and `raw` is a valid out-pointer. On success the shell hands us a
	// NUL-terminated wide string that we own and must free with CoTaskMemFree.
	let hr =
		unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT as u32, std::ptr::null_mut(), &mut raw) };
	if hr < 0 || raw.is_null() {
		// Per the docs, ppszPath is set to NULL on failure, so nothing to free.
		return None;
	}

	let path = unsafe {
		let mut len = 0usize;
		while *raw.add(len) != 0 {
			len += 1;
		}
		let wide = std::slice::from_raw_parts(raw, len);
		let os = OsString::from_wide(wide);
		CoTaskMemFree(raw.cast());
		PathBuf::from(os)
	};

	path.is_absolute().then_some(path)
}

// =============================================================================
// Directory implementations
// =============================================================================

pub fn home_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_Profile)
}

pub fn cache_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_LocalAppData)
}

pub fn config_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_RoamingAppData)
}

pub fn config_local_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_LocalAppData)
}

pub fn data_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_RoamingAppData)
}

pub fn data_local_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_LocalAppData)
}

pub fn executable_dir() -> Option<PathBuf> {
	None
}

pub fn preference_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_RoamingAppData)
}

pub fn runtime_dir() -> Option<PathBuf> {
	None
}

pub fn state_dir() -> Option<PathBuf> {
	None
}

pub fn audio_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_Music)
}

pub fn desktop_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_Desktop)
}

pub fn document_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_Documents)
}

pub fn download_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_Downloads)
}

pub fn font_dir() -> Option<PathBuf> {
	None
}

pub fn picture_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_Pictures)
}

pub fn public_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_Public)
}

pub fn template_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_Templates)
}

pub fn video_dir() -> Option<PathBuf> {
	known_folder(&FOLDERID_Videos)
}

// =============================================================================
// sysdirs extensions
// =============================================================================

/// `%TEMP%`, then `%TMP%` (absolute only), then `<LocalAppData>\Temp`.
pub fn temp_dir() -> Option<PathBuf> {
	std::env::var_os("TEMP")
		.or_else(|| std::env::var_os("TMP"))
		.map(PathBuf::from)
		.filter(|p| p.is_absolute())
		.or_else(|| cache_dir().map(|c| c.join("Temp")))
}

pub fn library_dir() -> Option<PathBuf> {
	None
}
