//! Environment-sensitive tests for Apple platforms (macOS test runner).
//!
//! Same child-process probe pattern as `xdg_env.rs`: the environment is
//! controlled by re-executing this binary, never by mutating our own.

#![cfg(target_os = "macos")]

use std::path::PathBuf;
use std::process::Command;

const PROBE_VAR: &str = "SYSDIRS_PROBE";

#[test]
fn probe() {
	let Ok(name) = std::env::var(PROBE_VAR) else {
		return;
	};
	let result = match name.as_str() {
		"home_dir" => sysdirs::home_dir(),
		"cache_dir" => sysdirs::cache_dir(),
		"config_dir" => sysdirs::config_dir(),
		"library_dir" => sysdirs::library_dir(),
		"temp_dir" => sysdirs::temp_dir(),
		other => panic!("unknown probe {other}"),
	};
	match result {
		Some(p) => println!("PROBE:Some:{}", p.display()),
		None => println!("PROBE:None"),
	}
}

/// Run `func` in a child that inherits our environment, then applies `env`.
fn lookup(func: &str, env: &[(&str, Option<&str>)]) -> Option<PathBuf> {
	let mut cmd = Command::new(std::env::current_exe().unwrap());
	cmd.args(["--exact", "probe", "--nocapture", "--test-threads=1"])
		.env(PROBE_VAR, func);
	for (k, v) in env {
		match v {
			Some(v) => {
				cmd.env(k, v);
			}
			None => {
				cmd.env_remove(k);
			}
		}
	}
	let out = cmd.output().expect("spawn probe");
	assert!(
		out.status.success(),
		"probe failed: {}",
		String::from_utf8_lossy(&out.stderr)
	);
	let stdout = String::from_utf8(out.stdout).unwrap();
	let start = stdout
		.find("PROBE:")
		.unwrap_or_else(|| panic!("no PROBE marker in:\n{stdout}"));
	let line = stdout[start..].lines().next().unwrap();
	match line.strip_prefix("PROBE:Some:") {
		Some(p) => Some(PathBuf::from(p)),
		None => {
			assert_eq!(line, "PROBE:None", "unexpected marker in:\n{stdout}");
			None
		}
	}
}

// -----------------------------------------------------------------------------
// temp_dir
// -----------------------------------------------------------------------------

#[test]
fn temp_dir_honors_absolute_tmpdir() {
	assert_eq!(
		lookup("temp_dir", &[("TMPDIR", Some("/private/tmp/custom"))]),
		Some(PathBuf::from("/private/tmp/custom"))
	);
}

#[test]
fn temp_dir_resolves_without_tmpdir() {
	// launchd agents, XPC services and sandboxed helpers run without TMPDIR.
	let dir = lookup("temp_dir", &[("TMPDIR", None)]).expect("temp_dir must not be None");
	assert!(dir.is_absolute(), "{dir:?}");
	assert!(dir.exists(), "{dir:?} should exist");
	// Darwin's per-user temp lives under /var/folders; /tmp is the fallback.
	assert!(
		dir.starts_with("/var/folders")
			|| dir.starts_with("/private/var/folders")
			|| dir == std::path::Path::new("/tmp"),
		"unexpected temp dir {dir:?}"
	);
}

#[test]
fn temp_dir_ignores_relative_tmpdir() {
	let dir = lookup("temp_dir", &[("TMPDIR", Some("tmp"))]).unwrap();
	assert!(dir.is_absolute(), "{dir:?}");
	assert_ne!(dir, PathBuf::from("tmp"));
}

#[test]
fn temp_dir_ignores_empty_tmpdir() {
	let dir = lookup("temp_dir", &[("TMPDIR", Some(""))]).unwrap();
	assert!(dir.is_absolute(), "{dir:?}");
}

// -----------------------------------------------------------------------------
// HOME
// -----------------------------------------------------------------------------

#[test]
fn home_dir_none_without_home() {
	assert_eq!(lookup("home_dir", &[("HOME", None)]), None);
}

#[test]
fn user_domain_dirs_none_without_home() {
	// sysdir returns "~/Library/..." for the user domain; with no HOME there
	// is nothing to expand it against.
	assert_eq!(lookup("cache_dir", &[("HOME", None)]), None);
	assert_eq!(lookup("config_dir", &[("HOME", None)]), None);
	assert_eq!(lookup("library_dir", &[("HOME", None)]), None);
}

#[test]
fn user_domain_dirs_follow_home_override() {
	let home = "/Users/probe";
	assert_eq!(
		lookup("cache_dir", &[("HOME", Some(home))]),
		Some(PathBuf::from("/Users/probe/Library/Caches"))
	);
	assert_eq!(
		lookup("config_dir", &[("HOME", Some(home))]),
		Some(PathBuf::from("/Users/probe/Library/Application Support"))
	);
	assert_eq!(
		lookup("library_dir", &[("HOME", Some(home))]),
		Some(PathBuf::from("/Users/probe/Library"))
	);
}
