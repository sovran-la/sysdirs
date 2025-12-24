# sysdirs

[![crates.io](https://img.shields.io/crates/v/sysdirs.svg?style=for-the-badge)](https://crates.io/crates/sysdirs)
[![docs.rs](https://img.shields.io/docsrs/sysdirs/latest?style=for-the-badge)](https://docs.rs/sysdirs/)
[![License: MIT](https://img.shields.io/badge/license-MIT-orange.svg?style=for-the-badge)](LICENSE)

## Introduction

* a tiny low-level library with a minimal API
* that provides the platform-specific, user-accessible locations
* for retrieving and storing configuration, cache and other data
* on Linux, Windows (≥ Vista), macOS, **iOS, Android, and WASM**

The library provides the location of these directories by leveraging the mechanisms defined by:

* the [XDG base directory](https://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html) and
  the [XDG user directory](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/) specifications on Linux
* the [Known Folder](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx) API on Windows
* the [Standard Directories](https://developer.apple.com/library/content/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemOverview/FileSystemOverview.html#//apple_ref/doc/uid/TP40010672-CH2-SW6) guidelines on macOS and iOS
* the app sandbox directories on Android (requires initialization)

## Why sysdirs?

The `dirs` crate is great, but it doesn't support mobile platforms. If you're building cross-platform Rust code that needs to run on iOS or Android, you're out of luck. `sysdirs` fills that gap with the same simple API, plus mobile support.

## Platforms

| Platform | Support |
|----------|:--------|
| Linux | ✅ Full |
| macOS | ✅ Full |
| Windows | ✅ Full |
| iOS/tvOS/watchOS/visionOS | ✅ Full |
| Android | ✅ Full (with `android-auto` or `init_android()`) |
| WASM | ⚠️ Compiles, but returns `None` |
| Other Unix (FreeBSD, Redox, etc.) | ✅ XDG fallback (FreeBSD CI tested) |

## Usage

Add the library as a dependency:

```toml
[dependencies]
sysdirs = "0.1"
```

### Example

```rust
use sysdirs;

sysdirs::home_dir();
// Lin: Some(/home/alice)
// Win: Some(C:\Users\Alice)
// Mac: Some(/Users/Alice)
// iOS: Some(/var/mobile/Containers/Data/Application/<UUID>)

sysdirs::cache_dir();
// Lin: Some(/home/alice/.cache)
// Win: Some(C:\Users\Alice\AppData\Local)
// Mac: Some(/Users/Alice/Library/Caches)
// iOS: Some(<sandbox>/Library/Caches)

sysdirs::config_dir();
// Lin: Some(/home/alice/.config)
// Win: Some(C:\Users\Alice\AppData\Roaming)
// Mac: Some(/Users/Alice/Library/Application Support)
// iOS: Some(<sandbox>/Library/Application Support)
```

### Common Patterns

The `PathExt` trait adds chainable operations for common tasks:

```rust
use sysdirs::PathExt;

// Chain path joins
let db_path = sysdirs::data_dir()
    .join("my-app")
    .join("database.sqlite");
// Linux: Some(/home/alice/.local/share/my-app/database.sqlite)

// Create app directory if it doesn't exist
let app_cache = sysdirs::cache_dir()
    .join("my-app")
    .ensure()?;  // Creates the directory, returns io::Result<PathBuf>

// One-liner for app config directory
let config = sysdirs::config_dir().join("my-app").ensure()?;
```

Without `PathExt`, checking if a path exists:

```rust
// Only proceed if the directory exists
if let Some(path) = sysdirs::data_dir().filter(|p| p.exists()) {
    // use path
}
```

### Android Setup

Android apps run in a sandbox and don't have environment variables pointing to their directories. There are two ways to use sysdirs on Android:

#### Option 1: Pure Rust Android apps

If you're building a pure Rust Android app using `android-activity` or `ndk-glue`, enable the `android-auto` feature:

```toml
[dependencies]
sysdirs = { version = "0.1", features = ["android-auto"] }
```

Paths are detected automatically via `ndk-context`. No initialization needed.

#### Option 2: Kotlin/Java apps embedding Rust

If you're embedding Rust in an existing Kotlin/Java app, call `init_android()` once at startup:

```kotlin
// In your Kotlin/Java code
external fun initSysdirs(filesDir: String)

// Call at app startup
initSysdirs(context.filesDir.absolutePath)
```

```rust
// In your Rust JNI code
#[no_mangle]
pub extern "C" fn Java_com_example_App_initSysdirs(
    env: JNIEnv,
    _: JClass,
    files_dir: JString,
) {
    let path: String = env.get_string(files_dir).unwrap().into();
    sysdirs::init_android(&path);
}
```

After initialization, all directory functions work normally:

```rust
sysdirs::cache_dir();  // Some(/data/data/com.example.app/files/cache)
sysdirs::config_dir(); // Some(/data/data/com.example.app/files)
```

## Directory Functions

| Function | Linux | macOS | Windows | iOS | Android* | WASM |
|----------|-------|-------|---------|-----|----------|------|
| `home_dir` | `$HOME` | `$HOME` | `{FOLDERID_Profile}` | sandbox | filesDir | None |
| `cache_dir` | `$XDG_CACHE_HOME` | `~/Library/Caches` | `{FOLDERID_LocalAppData}` | `Library/Caches` | `filesDir/cache` | None |
| `config_dir` | `$XDG_CONFIG_HOME` | `~/Library/Application Support` | `{FOLDERID_RoamingAppData}` | `Library/Application Support` | filesDir | None |
| `config_local_dir` | `$XDG_CONFIG_HOME` | `~/Library/Application Support` | `{FOLDERID_LocalAppData}` | `Library/Application Support` | filesDir | None |
| `data_dir` | `$XDG_DATA_HOME` | `~/Library/Application Support` | `{FOLDERID_RoamingAppData}` | `Library/Application Support` | filesDir | None |
| `data_local_dir` | `$XDG_DATA_HOME` | `~/Library/Application Support` | `{FOLDERID_LocalAppData}` | `Library/Application Support` | filesDir | None |
| `executable_dir` | `$XDG_BIN_HOME` | None | None | None | None | None |
| `preference_dir` | `$XDG_CONFIG_HOME` | `~/Library/Preferences` | `{FOLDERID_RoamingAppData}` | `Library/Preferences` | filesDir | None |
| `runtime_dir` | `$XDG_RUNTIME_DIR` | None | None | None | None | None |
| `state_dir` | `$XDG_STATE_HOME` | None | None | None | None | None |
| `audio_dir` | `XDG_MUSIC_DIR` | `~/Music` | `{FOLDERID_Music}` | None | None | None |
| `desktop_dir` | `XDG_DESKTOP_DIR` | `~/Desktop` | `{FOLDERID_Desktop}` | None | None | None |
| `document_dir` | `XDG_DOCUMENTS_DIR` | `~/Documents` | `{FOLDERID_Documents}` | `Documents` | None | None |
| `download_dir` | `XDG_DOWNLOAD_DIR` | `~/Downloads` | `{FOLDERID_Downloads}` | None | None | None |
| `font_dir` | `$XDG_DATA_HOME/fonts` | `~/Library/Fonts` | None | None | None | None |
| `picture_dir` | `XDG_PICTURES_DIR` | `~/Pictures` | `{FOLDERID_Pictures}` | None | None | None |
| `public_dir` | `XDG_PUBLICSHARE_DIR` | `~/Public` | `{FOLDERID_Public}` | None | None | None |
| `template_dir` | `XDG_TEMPLATES_DIR` | None | `{FOLDERID_Templates}` | None | None | None |
| `video_dir` | `XDG_VIDEOS_DIR` | `~/Movies` | `{FOLDERID_Videos}` | None | None | None |

\* Android requires either the `android-auto` feature or `init_android()` to be called first

### sysdirs Extensions

These functions are not present in the `dirs` crate:

| Function | Linux | macOS | Windows | iOS | Android* | WASM |
|----------|-------|-------|---------|-----|----------|------|
| `temp_dir` | `$TMPDIR` or `/tmp` | `$TMPDIR` | `%TEMP%` | `tmp` | `filesDir/tmp` | None |
| `library_dir` | None | `~/Library` | None | `Library` | None | None |

## Comparison with `dirs`

| Feature | dirs | sysdirs |
|---------|------|---------|
| Linux | ✅ | ✅ |
| macOS | ✅ | ✅ |
| Windows | ✅ | ✅ |
| Other Unix | ✅ | ✅ (XDG fallback) |
| **iOS** | ❌ | ✅ |
| **Android** | ❌ | ✅ |
| **WASM** | ❌ | ✅ (returns None) |
| API compatible | - | ✅ |
| `temp_dir()` | ❌ | ✅ |
| `library_dir()` | ❌ | ✅ |
| `PathExt` trait | ❌ | ✅ |

## Cargo Features

| Feature | Description |
|---------|-------------|
| (default) | Zero dependencies, manual Android init |
| `android-auto` | Auto-detect Android paths via `ndk-context` |

## Design Goals

* **Mobile-first**: iOS and Android are first-class citizens, not afterthoughts
* **Drop-in replacement**: Same API as `dirs` for easy migration
* **No dependencies**: Pure Rust, no C dependencies (except platform APIs)
* **Composable**: `PathExt` trait adds chainable `.join()` and `.ensure()` for common workflows

## License

MIT License - see [LICENSE](LICENSE) for details.
