//! Tests for Apple search path domain switching.
//!
//! The domain is process-wide, so every test that touches it holds `DomainGuard`,
//! which serializes the tests through a mutex and resets the domain to `User` on
//! drop (including on panic, since the guard is dropped during unwinding).

#![cfg(target_os = "macos")]

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use sysdirs::{SearchPathDomain, domain, set_domain};

static LOCK: Mutex<()> = Mutex::new(());

struct DomainGuard(#[allow(dead_code)] MutexGuard<'static, ()>);

impl DomainGuard {
	fn acquire() -> Self {
		// A poisoned lock just means an earlier test panicked; the state is
		// still reset by that test's guard, so it's safe to continue.
		let guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
		set_domain(SearchPathDomain::User);
		Self(guard)
	}
}

impl Drop for DomainGuard {
	fn drop(&mut self) {
		set_domain(SearchPathDomain::User);
	}
}

fn home() -> PathBuf {
	PathBuf::from(std::env::var("HOME").expect("HOME not set"))
}

// -----------------------------------------------------------------------------
// Getter / setter
// -----------------------------------------------------------------------------

#[test]
fn default_domain_is_user() {
	let _g = DomainGuard::acquire();
	assert_eq!(domain(), SearchPathDomain::User);
	assert_eq!(SearchPathDomain::default(), SearchPathDomain::User);
}

#[test]
fn domain_round_trips() {
	let _g = DomainGuard::acquire();
	for d in [
		SearchPathDomain::Local,
		SearchPathDomain::Network,
		SearchPathDomain::System,
		SearchPathDomain::User,
	] {
		set_domain(d);
		assert_eq!(domain(), d);
	}
}

#[test]
fn domain_is_process_wide() {
	let _g = DomainGuard::acquire();
	set_domain(SearchPathDomain::Local);

	// A fresh thread sees the domain set by this one.
	let (seen, cache) = std::thread::spawn(|| (domain(), sysdirs::cache_dir()))
		.join()
		.unwrap();
	assert_eq!(seen, SearchPathDomain::Local);
	assert_eq!(cache, Some(PathBuf::from("/Library/Caches")));

	// And a change made on another thread is visible here.
	std::thread::spawn(|| set_domain(SearchPathDomain::System))
		.join()
		.unwrap();
	assert_eq!(domain(), SearchPathDomain::System);
	assert_eq!(
		sysdirs::cache_dir(),
		Some(PathBuf::from("/System/Library/Caches"))
	);
}

// -----------------------------------------------------------------------------
// Per-domain paths
// -----------------------------------------------------------------------------

#[test]
fn user_domain_returns_home_paths() {
	let _g = DomainGuard::acquire();
	assert_eq!(sysdirs::cache_dir(), Some(home().join("Library/Caches")));
	assert_eq!(
		sysdirs::config_dir(),
		Some(home().join("Library/Application Support"))
	);
	assert_eq!(sysdirs::library_dir(), Some(home().join("Library")));
	assert_eq!(
		sysdirs::preference_dir(),
		Some(home().join("Library/Preferences"))
	);
	assert_eq!(sysdirs::font_dir(), Some(home().join("Library/Fonts")));
}

#[test]
fn aliases_are_consistent_in_every_domain() {
	let _g = DomainGuard::acquire();
	for d in [
		SearchPathDomain::User,
		SearchPathDomain::Local,
		SearchPathDomain::System,
	] {
		set_domain(d);
		assert_eq!(sysdirs::config_local_dir(), sysdirs::config_dir(), "{d:?}");
		assert_eq!(sysdirs::data_local_dir(), sysdirs::data_dir(), "{d:?}");
		assert_eq!(sysdirs::data_dir(), sysdirs::config_dir(), "{d:?}");
	}
}

#[test]
fn local_domain_returns_root_library() {
	let _g = DomainGuard::acquire();
	set_domain(SearchPathDomain::Local);
	assert_eq!(sysdirs::cache_dir(), Some(PathBuf::from("/Library/Caches")));
	assert_eq!(
		sysdirs::config_dir(),
		Some(PathBuf::from("/Library/Application Support"))
	);
	assert_eq!(sysdirs::library_dir(), Some(PathBuf::from("/Library")));
	assert_eq!(
		sysdirs::preference_dir(),
		Some(PathBuf::from("/Library/Preferences"))
	);
	assert_eq!(sysdirs::font_dir(), Some(PathBuf::from("/Library/Fonts")));
}

#[test]
fn system_domain_returns_system_library() {
	let _g = DomainGuard::acquire();
	set_domain(SearchPathDomain::System);
	assert_eq!(
		sysdirs::cache_dir(),
		Some(PathBuf::from("/System/Library/Caches"))
	);
	assert_eq!(
		sysdirs::library_dir(),
		Some(PathBuf::from("/System/Library"))
	);
}

#[test]
fn user_only_dirs_are_none_outside_user_domain() {
	// Documents, Desktop, Downloads etc. only exist per-user. Outside the
	// user domain sysdir has no entry for them.
	let _g = DomainGuard::acquire();
	for d in [
		SearchPathDomain::Local,
		SearchPathDomain::Network,
		SearchPathDomain::System,
	] {
		set_domain(d);
		assert_eq!(sysdirs::document_dir(), None, "{d:?}");
		assert_eq!(sysdirs::desktop_dir(), None, "{d:?}");
		assert_eq!(sysdirs::download_dir(), None, "{d:?}");
	}
}

#[test]
fn network_domain_returns_network_library() {
	let _g = DomainGuard::acquire();
	set_domain(SearchPathDomain::Network);
	assert_eq!(
		sysdirs::library_dir(),
		Some(PathBuf::from("/Network/Library"))
	);
}

#[test]
fn domain_switch_is_immediate_and_reversible() {
	let _g = DomainGuard::acquire();
	let user_cache = sysdirs::cache_dir().unwrap();

	set_domain(SearchPathDomain::Local);
	let local_cache = sysdirs::cache_dir().unwrap();

	set_domain(SearchPathDomain::User);
	let user_cache_again = sysdirs::cache_dir().unwrap();

	assert_ne!(user_cache, local_cache);
	assert_eq!(user_cache, user_cache_again);
	assert_eq!(local_cache, PathBuf::from("/Library/Caches"));
}

// -----------------------------------------------------------------------------
// Things the domain must not affect
// -----------------------------------------------------------------------------

#[test]
fn home_and_temp_ignore_domain() {
	let _g = DomainGuard::acquire();
	let home_user = sysdirs::home_dir();
	let temp_user = sysdirs::temp_dir();

	for d in [
		SearchPathDomain::Local,
		SearchPathDomain::Network,
		SearchPathDomain::System,
	] {
		set_domain(d);
		assert_eq!(sysdirs::home_dir(), home_user, "{d:?}");
		assert_eq!(sysdirs::temp_dir(), temp_user, "{d:?}");
	}
	assert_eq!(home_user, Some(home()));
}

#[test]
fn unsupported_dirs_are_none_in_every_domain() {
	let _g = DomainGuard::acquire();
	for d in [
		SearchPathDomain::User,
		SearchPathDomain::Local,
		SearchPathDomain::Network,
		SearchPathDomain::System,
	] {
		set_domain(d);
		assert_eq!(sysdirs::executable_dir(), None, "{d:?}");
		assert_eq!(sysdirs::runtime_dir(), None, "{d:?}");
		assert_eq!(sysdirs::state_dir(), None, "{d:?}");
		assert_eq!(sysdirs::template_dir(), None, "{d:?}");
	}
}

// -----------------------------------------------------------------------------
// User-media dirs (macOS only, user domain)
// -----------------------------------------------------------------------------

#[test]
fn user_media_dirs_live_under_home() {
	let _g = DomainGuard::acquire();
	let h = home();
	assert_eq!(sysdirs::audio_dir(), Some(h.join("Music")));
	assert_eq!(sysdirs::desktop_dir(), Some(h.join("Desktop")));
	assert_eq!(sysdirs::document_dir(), Some(h.join("Documents")));
	assert_eq!(sysdirs::download_dir(), Some(h.join("Downloads")));
	assert_eq!(sysdirs::picture_dir(), Some(h.join("Pictures")));
	assert_eq!(sysdirs::public_dir(), Some(h.join("Public")));
	assert_eq!(sysdirs::video_dir(), Some(h.join("Movies")));
}
