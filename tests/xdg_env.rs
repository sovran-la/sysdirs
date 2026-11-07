//! End-to-end XDG environment tests for Linux and the generic Unix fallback.
//!
//! Mutating the process environment is unsafe and racy across test threads,
//! so each case re-executes this test binary as a child with a controlled
//! environment and asks it to print one directory lookup. The child path is
//! the `probe` test below, selected with `--exact`.

#![cfg(not(any(
	target_os = "macos",
	target_os = "ios",
	target_os = "tvos",
	target_os = "watchos",
	target_os = "visionos",
	target_os = "windows",
	target_os = "android",
	target_arch = "wasm32"
)))]

use std::path::PathBuf;
use std::process::Command;

const PROBE_VAR: &str = "SYSDIRS_PROBE";
const HOME: &str = "/home/probe";

/// Child-side entry point. Not a real test: it only does anything when
/// `SYSDIRS_PROBE` names a function, and then prints that function's result.
#[test]
fn probe() {
	let Ok(name) = std::env::var(PROBE_VAR) else {
		return;
	};
	let result = match name.as_str() {
		"home_dir" => sysdirs::home_dir(),
		"cache_dir" => sysdirs::cache_dir(),
		"config_dir" => sysdirs::config_dir(),
		"data_dir" => sysdirs::data_dir(),
		"state_dir" => sysdirs::state_dir(),
		"executable_dir" => sysdirs::executable_dir(),
		"runtime_dir" => sysdirs::runtime_dir(),
		"audio_dir" => sysdirs::audio_dir(),
		"desktop_dir" => sysdirs::desktop_dir(),
		"video_dir" => sysdirs::video_dir(),
		"font_dir" => sysdirs::font_dir(),
		"temp_dir" => sysdirs::temp_dir(),
		other => panic!("unknown probe {other}"),
	};
	match result {
		Some(p) => println!("PROBE:Some:{}", p.display()),
		None => println!("PROBE:None"),
	}
}

/// Run `func` in a child with exactly `env` set (plus HOME unless overridden).
fn lookup(func: &str, env: &[(&str, Option<&str>)]) -> Option<PathBuf> {
	let mut cmd = Command::new(std::env::current_exe().unwrap());
	cmd.args(["--exact", "probe", "--nocapture", "--test-threads=1"])
		.env_clear()
		.env("HOME", HOME)
		.env(PROBE_VAR, func);
	// Keep coverage instrumentation working in the child.
	for (k, v) in std::env::vars_os() {
		if k.to_string_lossy().starts_with("LLVM_PROFILE_FILE") {
			cmd.env(k, v);
		}
	}
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
	// libtest prints "test probe ... " before our output on the same line,
	// so locate the marker by substring rather than line start.
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

fn p(s: &str) -> Option<PathBuf> {
	Some(PathBuf::from(s))
}

// -----------------------------------------------------------------------------
// HOME
// -----------------------------------------------------------------------------

#[test]
fn home_dir_reads_home() {
	assert_eq!(lookup("home_dir", &[]), p(HOME));
}

#[test]
fn home_dir_none_when_home_unset() {
	assert_eq!(lookup("home_dir", &[("HOME", None)]), None);
}

#[test]
fn home_dir_none_when_home_relative() {
	assert_eq!(lookup("home_dir", &[("HOME", Some("relative"))]), None);
}

#[test]
fn home_dir_none_when_home_empty() {
	assert_eq!(lookup("home_dir", &[("HOME", Some(""))]), None);
}

// -----------------------------------------------------------------------------
// Base dirs: valid override
// -----------------------------------------------------------------------------

#[test]
fn cache_dir_honors_absolute_override() {
	assert_eq!(
		lookup("cache_dir", &[("XDG_CACHE_HOME", Some("/custom/cache"))]),
		p("/custom/cache")
	);
}

#[test]
fn config_dir_honors_tilde_override() {
	assert_eq!(
		lookup("config_dir", &[("XDG_CONFIG_HOME", Some("~/cfg"))]),
		p("/home/probe/cfg")
	);
}

#[test]
fn data_dir_honors_absolute_override() {
	assert_eq!(
		lookup("data_dir", &[("XDG_DATA_HOME", Some("/srv/data"))]),
		p("/srv/data")
	);
}

#[test]
fn state_dir_honors_absolute_override() {
	assert_eq!(
		lookup("state_dir", &[("XDG_STATE_HOME", Some("/srv/state"))]),
		p("/srv/state")
	);
}

#[test]
fn executable_dir_honors_absolute_override() {
	assert_eq!(
		lookup("executable_dir", &[("XDG_BIN_HOME", Some("/opt/bin"))]),
		p("/opt/bin")
	);
}

// -----------------------------------------------------------------------------
// Base dirs: defaults
// -----------------------------------------------------------------------------

#[test]
fn base_dirs_default_under_home() {
	assert_eq!(lookup("cache_dir", &[]), p("/home/probe/.cache"));
	assert_eq!(lookup("config_dir", &[]), p("/home/probe/.config"));
	assert_eq!(lookup("data_dir", &[]), p("/home/probe/.local/share"));
	assert_eq!(lookup("state_dir", &[]), p("/home/probe/.local/state"));
	assert_eq!(lookup("executable_dir", &[]), p("/home/probe/.local/bin"));
}

#[test]
fn base_dirs_none_without_home() {
	let no_home = [("HOME", None)];
	assert_eq!(lookup("cache_dir", &no_home), None);
	assert_eq!(lookup("config_dir", &no_home), None);
	assert_eq!(lookup("data_dir", &no_home), None);
}

#[test]
fn base_dir_absolute_override_works_without_home() {
	assert_eq!(
		lookup(
			"cache_dir",
			&[("HOME", None), ("XDG_CACHE_HOME", Some("/custom"))]
		),
		p("/custom")
	);
}

// -----------------------------------------------------------------------------
// Base dirs: invalid override is ignored (spec: must be absolute)
// -----------------------------------------------------------------------------

#[test]
fn cache_dir_ignores_empty_override() {
	assert_eq!(
		lookup("cache_dir", &[("XDG_CACHE_HOME", Some(""))]),
		p("/home/probe/.cache")
	);
}

#[test]
fn config_dir_ignores_relative_override() {
	assert_eq!(
		lookup("config_dir", &[("XDG_CONFIG_HOME", Some("cfg"))]),
		p("/home/probe/.config")
	);
}

#[test]
fn data_dir_ignores_traversal_override() {
	assert_eq!(
		lookup("data_dir", &[("XDG_DATA_HOME", Some("../../etc"))]),
		p("/home/probe/.local/share")
	);
}

#[test]
fn config_dir_ignores_tilde_override_without_home() {
	assert_eq!(
		lookup(
			"config_dir",
			&[("HOME", None), ("XDG_CONFIG_HOME", Some("~/cfg"))]
		),
		None
	);
}

#[test]
fn config_dir_ignores_non_utf8_override() {
	use std::os::unix::ffi::OsStrExt;
	let bad = std::ffi::OsStr::from_bytes(b"/bad/\xff\xfe");
	let mut cmd = Command::new(std::env::current_exe().unwrap());
	cmd.args(["--exact", "probe", "--nocapture", "--test-threads=1"])
		.env_clear()
		.env("HOME", HOME)
		.env(PROBE_VAR, "config_dir")
		.env("XDG_CONFIG_HOME", bad);
	if let Some(v) = std::env::var_os("LLVM_PROFILE_FILE") {
		cmd.env("LLVM_PROFILE_FILE", v);
	}
	let out = cmd.output().unwrap();
	assert!(out.status.success());
	let stdout = String::from_utf8_lossy(&out.stdout);
	assert!(
		stdout.contains("PROBE:Some:/home/probe/.config"),
		"got:\n{stdout}"
	);
}

// -----------------------------------------------------------------------------
// Derived dirs
// -----------------------------------------------------------------------------

#[test]
fn font_dir_follows_data_dir_override() {
	assert_eq!(
		lookup("font_dir", &[("XDG_DATA_HOME", Some("/srv/data"))]),
		p("/srv/data/fonts")
	);
}

#[test]
fn font_dir_follows_data_dir_default() {
	assert_eq!(lookup("font_dir", &[]), p("/home/probe/.local/share/fonts"));
}

// -----------------------------------------------------------------------------
// Optional dirs (no default)
// -----------------------------------------------------------------------------

#[test]
fn runtime_dir_honors_absolute() {
	assert_eq!(
		lookup(
			"runtime_dir",
			&[("XDG_RUNTIME_DIR", Some("/run/user/1000"))]
		),
		p("/run/user/1000")
	);
}

#[test]
fn runtime_dir_none_when_unset() {
	assert_eq!(lookup("runtime_dir", &[("XDG_RUNTIME_DIR", None)]), None);
}

#[test]
fn runtime_dir_none_when_relative() {
	assert_eq!(
		lookup("runtime_dir", &[("XDG_RUNTIME_DIR", Some("run"))]),
		None
	);
}

#[test]
fn runtime_dir_none_when_empty() {
	assert_eq!(
		lookup("runtime_dir", &[("XDG_RUNTIME_DIR", Some(""))]),
		None
	);
}

// -----------------------------------------------------------------------------
// User dirs: env var, then $XDG_CONFIG_HOME/user-dirs.dirs
// -----------------------------------------------------------------------------

/// A scratch XDG_CONFIG_HOME holding one `user-dirs.dirs`, removed on drop.
struct ConfigHome(PathBuf);

impl ConfigHome {
	fn with_user_dirs(contents: &str) -> Self {
		let dir = std::env::temp_dir().join(format!(
			"sysdirs-xdg-env-{}-{:?}",
			std::process::id(),
			std::thread::current().id()
		));
		std::fs::create_dir_all(&dir).unwrap();
		std::fs::write(dir.join("user-dirs.dirs"), contents).unwrap();
		Self(dir)
	}

	fn path(&self) -> &str {
		self.0.to_str().unwrap()
	}
}

impl Drop for ConfigHome {
	fn drop(&mut self) {
		let _ = std::fs::remove_dir_all(&self.0);
	}
}

const USER_DIRS: &str = "XDG_MUSIC_DIR=\"$HOME/Music\"\nXDG_DESKTOP_DIR=\"$HOME/\"\n";

#[test]
fn user_dir_env_var_wins() {
	assert_eq!(
		lookup("audio_dir", &[("XDG_MUSIC_DIR", Some("/home/probe/Tunes"))]),
		p("/home/probe/Tunes")
	);
}

#[test]
fn user_dir_none_when_nothing_configured() {
	let cfg = ConfigHome::with_user_dirs("");
	assert_eq!(
		lookup("audio_dir", &[("XDG_CONFIG_HOME", Some(cfg.path()))]),
		None
	);
}

#[test]
fn user_dir_none_when_file_missing() {
	let cfg = ConfigHome::with_user_dirs("");
	std::fs::remove_file(cfg.0.join("user-dirs.dirs")).unwrap();
	assert_eq!(
		lookup("audio_dir", &[("XDG_CONFIG_HOME", Some(cfg.path()))]),
		None
	);
}

#[test]
fn user_dir_read_from_file() {
	let cfg = ConfigHome::with_user_dirs(USER_DIRS);
	assert_eq!(
		lookup("audio_dir", &[("XDG_CONFIG_HOME", Some(cfg.path()))]),
		p("/home/probe/Music")
	);
}

#[test]
fn user_dir_env_var_overrides_file() {
	let cfg = ConfigHome::with_user_dirs(USER_DIRS);
	assert_eq!(
		lookup(
			"audio_dir",
			&[
				("XDG_CONFIG_HOME", Some(cfg.path())),
				("XDG_MUSIC_DIR", Some("/srv/music"))
			]
		),
		p("/srv/music")
	);
}

#[test]
fn user_dir_invalid_env_var_falls_through_to_file() {
	let cfg = ConfigHome::with_user_dirs(USER_DIRS);
	assert_eq!(
		lookup(
			"audio_dir",
			&[
				("XDG_CONFIG_HOME", Some(cfg.path())),
				("XDG_MUSIC_DIR", Some("relative"))
			]
		),
		p("/home/probe/Music")
	);
}

#[test]
fn user_dir_disabled_in_file_is_none() {
	let cfg = ConfigHome::with_user_dirs(USER_DIRS);
	assert_eq!(
		lookup("desktop_dir", &[("XDG_CONFIG_HOME", Some(cfg.path()))]),
		None
	);
}

#[test]
fn user_dir_file_needs_home_for_home_relative_entries() {
	let cfg = ConfigHome::with_user_dirs(USER_DIRS);
	assert_eq!(
		lookup(
			"audio_dir",
			&[("HOME", None), ("XDG_CONFIG_HOME", Some(cfg.path()))]
		),
		None
	);
}

#[test]
fn user_dir_file_follows_default_config_home() {
	// No XDG_CONFIG_HOME: the file is looked up under $HOME/.config.
	let cfg = ConfigHome::with_user_dirs("");
	let home = cfg.0.join("home");
	std::fs::create_dir_all(home.join(".config")).unwrap();
	std::fs::write(
		home.join(".config/user-dirs.dirs"),
		"XDG_VIDEOS_DIR=\"$HOME/Movies\"\n",
	)
	.unwrap();
	assert_eq!(
		lookup(
			"video_dir",
			&[
				("HOME", Some(home.to_str().unwrap())),
				("XDG_CONFIG_HOME", None)
			]
		),
		Some(home.join("Movies"))
	);
}

// -----------------------------------------------------------------------------
// temp_dir
// -----------------------------------------------------------------------------

#[test]
fn temp_dir_honors_tmpdir() {
	assert_eq!(
		lookup("temp_dir", &[("TMPDIR", Some("/var/tmp"))]),
		p("/var/tmp")
	);
}

#[test]
fn temp_dir_defaults_to_tmp() {
	assert_eq!(lookup("temp_dir", &[("TMPDIR", None)]), p("/tmp"));
}

#[test]
fn temp_dir_ignores_relative_tmpdir() {
	assert_eq!(lookup("temp_dir", &[("TMPDIR", Some("tmp"))]), p("/tmp"));
}

#[test]
fn temp_dir_ignores_empty_tmpdir() {
	assert_eq!(lookup("temp_dir", &[("TMPDIR", Some(""))]), p("/tmp"));
}
