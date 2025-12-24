//! Tests for PathExt trait.

use std::path::PathBuf;
use sysdirs::PathExt;

#[test]
fn test_join_chains() {
	let path = Some(PathBuf::from("/base")).join("level1").join("level2");

	assert_eq!(path, Some(PathBuf::from("/base/level1/level2")));
}

#[test]
fn test_join_on_none() {
	let path: Option<PathBuf> = None;
	let result = path.join("something");

	assert_eq!(result, None);
}

#[test]
fn test_ensure_creates_directory() {
	let temp = std::env::temp_dir();
	let test_dir = temp.join("sysdirs-test-ensure");

	// Clean up from any previous run
	let _ = std::fs::remove_dir_all(&test_dir);

	// Ensure should create it
	let result = Some(test_dir.clone()).ensure();
	assert!(result.is_ok());
	assert_eq!(result.unwrap(), test_dir);
	assert!(test_dir.exists());

	// Clean up
	let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_ensure_on_none() {
	let path: Option<PathBuf> = None;
	let result = path.ensure();

	assert!(result.is_err());
	let err = result.unwrap_err();
	assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn test_join_then_ensure() {
	let temp = std::env::temp_dir();
	let test_dir = temp.join("sysdirs-test-join-ensure");

	// Clean up from any previous run
	let _ = std::fs::remove_dir_all(&test_dir);

	// Chain join and ensure
	let result = Some(temp.clone())
		.join("sysdirs-test-join-ensure")
		.join("nested")
		.join("deep")
		.ensure();

	assert!(result.is_ok());
	let path = result.unwrap();
	assert!(path.exists());
	assert!(path.ends_with("sysdirs-test-join-ensure/nested/deep"));

	// Clean up
	let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_real_dirs_with_pathext() {
	// Test with actual sysdirs functions
	let path = sysdirs::cache_dir().join("test-app");

	#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
	assert!(path.is_some());

	#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
	assert!(path.unwrap().ends_with("test-app"));
}
