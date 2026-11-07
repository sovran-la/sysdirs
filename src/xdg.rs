//! Shared XDG Base Directory logic for Linux and other Unix platforms.
//!
//! The XDG Base Directory specification requires every `XDG_*` value to be an
//! absolute path; empty or relative values must be treated as unset. We honor
//! that, with one extension: a leading `~/` is expanded against `$HOME` before
//! the absolute-path check, since that is what users overwhelmingly mean when
//! they write it into a config file.

use std::path::{Path, PathBuf};

// =============================================================================
// Core logic (testable, no env access)
// =============================================================================

/// Expand a leading tilde against `home`, then require the result to be absolute.
///
/// Returns `None` for an empty string, a relative path, a tilde form with no
/// home, or a tilde form whose home is itself relative.
pub(crate) fn validate_path(value: &str, home: Option<&Path>) -> Option<PathBuf> {
	if value.is_empty() {
		return None;
	}

	let path = if let Some(rest) = value.strip_prefix("~/") {
		home?.join(rest)
	} else if value == "~" {
		home?.to_path_buf()
	} else {
		PathBuf::from(value)
	};

	path.is_absolute().then_some(path)
}

/// Resolve a base directory: a valid env value wins, otherwise `home/default_suffix`.
pub(crate) fn resolve_base_dir(
	env_value: Option<&str>,
	home: Option<&Path>,
	default_suffix: &str,
) -> Option<PathBuf> {
	env_value
		.and_then(|v| validate_path(v, home))
		.or_else(|| home.map(|h| h.join(default_suffix)))
}

/// Resolve a directory that has no default: a valid env value or nothing.
pub(crate) fn resolve_optional_dir(
	env_value: Option<&str>,
	home: Option<&Path>,
) -> Option<PathBuf> {
	env_value.and_then(|v| validate_path(v, home))
}

/// Resolve the temp directory: a valid `$TMPDIR`, otherwise `/tmp`.
pub(crate) fn resolve_temp_dir(env_value: Option<&str>, home: Option<&Path>) -> Option<PathBuf> {
	env_value
		.and_then(|v| validate_path(v, home))
		.or_else(|| Some(PathBuf::from("/tmp")))
}

// =============================================================================
// Env var wrappers
// =============================================================================

pub(crate) fn home() -> Option<PathBuf> {
	std::env::var_os("HOME")
		.map(PathBuf::from)
		.filter(|h| h.is_absolute())
}

fn env(var: &str) -> Option<String> {
	std::env::var(var).ok()
}

pub(crate) fn base_dir(env_var: &str, default_suffix: &str) -> Option<PathBuf> {
	let home = home();
	resolve_base_dir(env(env_var).as_deref(), home.as_deref(), default_suffix)
}

pub(crate) fn optional_dir(env_var: &str) -> Option<PathBuf> {
	let home = home();
	resolve_optional_dir(env(env_var).as_deref(), home.as_deref())
}

pub(crate) fn temp_dir() -> Option<PathBuf> {
	let home = home();
	resolve_temp_dir(env("TMPDIR").as_deref(), home.as_deref())
}

// =============================================================================
// Tests (parallel-safe, no env manipulation)
// =============================================================================

#[cfg(test)]
mod tests {
	use super::*;

	const HOME: &str = "/home/testuser";

	fn home() -> Option<&'static Path> {
		Some(Path::new(HOME))
	}

	// -------------------------------------------------------------------------
	// validate_path: accepted forms
	// -------------------------------------------------------------------------

	#[test]
	fn absolute_path_is_accepted() {
		assert_eq!(
			validate_path("/absolute/path", home()),
			Some(PathBuf::from("/absolute/path"))
		);
	}

	#[test]
	fn absolute_path_is_accepted_without_home() {
		assert_eq!(
			validate_path("/absolute/path", None),
			Some(PathBuf::from("/absolute/path"))
		);
	}

	#[test]
	fn root_is_accepted() {
		assert_eq!(validate_path("/", home()), Some(PathBuf::from("/")));
	}

	#[test]
	fn tilde_slash_expands() {
		assert_eq!(
			validate_path("~/my-cache", home()),
			Some(PathBuf::from("/home/testuser/my-cache"))
		);
	}

	#[test]
	fn tilde_slash_expands_nested() {
		assert_eq!(
			validate_path("~/foo/bar/baz", home()),
			Some(PathBuf::from("/home/testuser/foo/bar/baz"))
		);
	}

	#[test]
	fn bare_tilde_expands() {
		assert_eq!(validate_path("~", home()), Some(PathBuf::from(HOME)));
	}

	#[test]
	fn absolute_path_with_trailing_slash_is_accepted() {
		assert_eq!(
			validate_path("/var/cache/", home()),
			Some(PathBuf::from("/var/cache/"))
		);
	}

	#[test]
	fn absolute_path_with_spaces_is_accepted() {
		assert_eq!(
			validate_path("/home/test user/.config", home()),
			Some(PathBuf::from("/home/test user/.config"))
		);
	}

	// -------------------------------------------------------------------------
	// validate_path: rejected forms (spec: non-absolute is invalid)
	// -------------------------------------------------------------------------

	#[test]
	fn empty_is_rejected() {
		assert_eq!(validate_path("", home()), None);
	}

	#[test]
	fn relative_path_is_rejected() {
		assert_eq!(validate_path("foo", home()), None);
	}

	#[test]
	fn relative_nested_path_is_rejected() {
		assert_eq!(validate_path("foo/bar", home()), None);
	}

	#[test]
	fn dot_is_rejected() {
		assert_eq!(validate_path(".", home()), None);
	}

	#[test]
	fn dot_slash_is_rejected() {
		assert_eq!(validate_path("./config", home()), None);
	}

	#[test]
	fn parent_traversal_is_rejected() {
		assert_eq!(validate_path("../../etc", home()), None);
	}

	#[test]
	fn whitespace_only_is_rejected() {
		assert_eq!(validate_path("   ", home()), None);
	}

	#[test]
	fn tilde_in_middle_is_relative_and_rejected() {
		assert_eq!(validate_path("some/~/path", home()), None);
	}

	#[test]
	fn tilde_user_form_is_rejected() {
		// We don't resolve other users' homes; `~bob/x` is a relative path.
		assert_eq!(validate_path("~bob/config", home()), None);
	}

	#[test]
	fn tilde_without_separator_is_rejected() {
		assert_eq!(validate_path("~foo", home()), None);
	}

	#[test]
	fn tilde_slash_without_home_is_rejected() {
		assert_eq!(validate_path("~/my-cache", None), None);
	}

	#[test]
	fn bare_tilde_without_home_is_rejected() {
		assert_eq!(validate_path("~", None), None);
	}

	#[test]
	fn tilde_with_relative_home_is_rejected() {
		let relative_home = Path::new("relative/home");
		assert_eq!(validate_path("~/x", Some(relative_home)), None);
		assert_eq!(validate_path("~", Some(relative_home)), None);
	}

	// -------------------------------------------------------------------------
	// resolve_base_dir
	// -------------------------------------------------------------------------

	#[test]
	fn base_dir_uses_valid_env_value() {
		assert_eq!(
			resolve_base_dir(Some("/custom/cache"), home(), ".cache"),
			Some(PathBuf::from("/custom/cache"))
		);
	}

	#[test]
	fn base_dir_expands_tilde_env_value() {
		assert_eq!(
			resolve_base_dir(Some("~/my-cache"), home(), ".cache"),
			Some(PathBuf::from("/home/testuser/my-cache"))
		);
	}

	#[test]
	fn base_dir_falls_back_when_unset() {
		assert_eq!(
			resolve_base_dir(None, home(), ".cache"),
			Some(PathBuf::from("/home/testuser/.cache"))
		);
	}

	#[test]
	fn base_dir_falls_back_when_empty() {
		assert_eq!(
			resolve_base_dir(Some(""), home(), ".cache"),
			Some(PathBuf::from("/home/testuser/.cache"))
		);
	}

	#[test]
	fn base_dir_falls_back_when_relative() {
		assert_eq!(
			resolve_base_dir(Some("foo"), home(), ".cache"),
			Some(PathBuf::from("/home/testuser/.cache"))
		);
	}

	#[test]
	fn base_dir_falls_back_when_traversal() {
		assert_eq!(
			resolve_base_dir(Some("../../etc"), home(), ".config"),
			Some(PathBuf::from("/home/testuser/.config"))
		);
	}

	#[test]
	fn base_dir_none_when_no_home_and_no_env() {
		assert_eq!(resolve_base_dir(None, None, ".cache"), None);
	}

	#[test]
	fn base_dir_none_when_no_home_and_invalid_env() {
		assert_eq!(resolve_base_dir(Some("foo"), None, ".cache"), None);
		assert_eq!(resolve_base_dir(Some(""), None, ".cache"), None);
	}

	#[test]
	fn base_dir_absolute_env_works_without_home() {
		assert_eq!(
			resolve_base_dir(Some("/custom"), None, ".cache"),
			Some(PathBuf::from("/custom"))
		);
	}

	#[test]
	fn base_dir_tilde_env_without_home_falls_through_to_none() {
		assert_eq!(resolve_base_dir(Some("~/x"), None, ".cache"), None);
	}

	#[test]
	fn base_dir_defaults_for_each_spec_var() {
		let cases = [
			(".cache", "/home/testuser/.cache"),
			(".config", "/home/testuser/.config"),
			(".local/share", "/home/testuser/.local/share"),
			(".local/state", "/home/testuser/.local/state"),
			(".local/bin", "/home/testuser/.local/bin"),
		];
		for (suffix, expected) in cases {
			assert_eq!(
				resolve_base_dir(None, home(), suffix),
				Some(PathBuf::from(expected)),
				"suffix {suffix}"
			);
		}
	}

	// -------------------------------------------------------------------------
	// resolve_optional_dir
	// -------------------------------------------------------------------------

	#[test]
	fn optional_dir_uses_valid_env_value() {
		assert_eq!(
			resolve_optional_dir(Some("/home/testuser/Music"), home()),
			Some(PathBuf::from("/home/testuser/Music"))
		);
	}

	#[test]
	fn optional_dir_expands_tilde() {
		assert_eq!(
			resolve_optional_dir(Some("~/Music"), home()),
			Some(PathBuf::from("/home/testuser/Music"))
		);
	}

	#[test]
	fn optional_dir_none_when_unset() {
		assert_eq!(resolve_optional_dir(None, home()), None);
	}

	#[test]
	fn optional_dir_none_when_empty() {
		assert_eq!(resolve_optional_dir(Some(""), home()), None);
	}

	#[test]
	fn optional_dir_none_when_relative() {
		assert_eq!(resolve_optional_dir(Some("Music"), home()), None);
	}

	#[test]
	fn optional_dir_absolute_works_without_home() {
		assert_eq!(
			resolve_optional_dir(Some("/run/user/1000"), None),
			Some(PathBuf::from("/run/user/1000"))
		);
	}

	// -------------------------------------------------------------------------
	// resolve_temp_dir
	// -------------------------------------------------------------------------

	#[test]
	fn temp_dir_uses_valid_tmpdir() {
		assert_eq!(
			resolve_temp_dir(Some("/var/tmp"), home()),
			Some(PathBuf::from("/var/tmp"))
		);
	}

	#[test]
	fn temp_dir_expands_tilde_tmpdir() {
		assert_eq!(
			resolve_temp_dir(Some("~/tmp"), home()),
			Some(PathBuf::from("/home/testuser/tmp"))
		);
	}

	#[test]
	fn temp_dir_defaults_to_tmp_when_unset() {
		assert_eq!(resolve_temp_dir(None, home()), Some(PathBuf::from("/tmp")));
	}

	#[test]
	fn temp_dir_defaults_to_tmp_when_empty() {
		assert_eq!(
			resolve_temp_dir(Some(""), home()),
			Some(PathBuf::from("/tmp"))
		);
	}

	#[test]
	fn temp_dir_defaults_to_tmp_when_relative() {
		assert_eq!(
			resolve_temp_dir(Some("tmp"), home()),
			Some(PathBuf::from("/tmp"))
		);
	}

	#[test]
	fn temp_dir_defaults_to_tmp_without_home() {
		assert_eq!(resolve_temp_dir(None, None), Some(PathBuf::from("/tmp")));
		assert_eq!(
			resolve_temp_dir(Some("~/tmp"), None),
			Some(PathBuf::from("/tmp"))
		);
	}
}
