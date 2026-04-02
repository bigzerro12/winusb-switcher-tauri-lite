//! Platform abstraction for PATH management and JLink search directories.

use std::path::PathBuf;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "linux")]
pub mod linux;

pub struct PlatformConfig {
    pub jlink_bin: &'static str,
    pub jlink_executable: &'static str,
}

pub fn config() -> PlatformConfig {
    #[cfg(target_os = "windows")]
    return PlatformConfig {
        jlink_bin: "JLink",
        jlink_executable: "JLink.exe",
    };
    #[cfg(target_os = "macos")]
    return PlatformConfig {
        jlink_bin: "JLinkExe",
        jlink_executable: "JLinkExe",
    };
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return PlatformConfig {
        jlink_bin: "JLinkExe",
        jlink_executable: "JLinkExe",
    };
}

pub fn search_dirs() -> Vec<PathBuf> {
    #[cfg(target_os = "windows")]
    return windows::search_dirs();
    #[cfg(target_os = "macos")]
    return macos::search_dirs();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return linux::search_dirs();
}

/// Find directory containing JLink executable in known locations.
pub fn find_jlink_in_search_dirs() -> Option<PathBuf> {
    let executable = config().jlink_executable;
    for base in search_dirs() {
        if !base.exists() { continue; }
        if let Ok(entries) = std::fs::read_dir(&base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join(executable).exists() {
                    return Some(path);
                }
            }
        }
        if base.join(executable).exists() {
            return Some(base);
        }
    }
    None
}

/// Update current process PATH so the new dir is usable in this session.
pub fn prepend_to_process_path(dir: &str) {
    let path_key = std::env::vars()
        .find(|(k, _)| k.to_lowercase() == "path")
        .map(|(k, _)| k)
        .unwrap_or_else(|| "PATH".to_string());

    let current = std::env::var(&path_key).unwrap_or_default();
    if !current.to_lowercase().contains(&dir.to_lowercase()) {
        let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
        // Prepend so our bundled J-Link wins over any stale PATH entries.
        std::env::set_var(&path_key, format!("{}{}{}", dir, separator, current));
    }
}

/// Linux: SEGGER `JLinkExe` loads `libjlinkarm.so` from the install directory. If only `PATH` is set,
/// the dynamic linker may still fail with **"Could not open J-Link shared library"** unless the
/// directory is also on `LD_LIBRARY_PATH` (RPATH/`$ORIGIN` can be insufficient in some layouts).
#[cfg(target_os = "linux")]
pub fn prepend_ld_library_path(dir: &str) {
    const KEY: &str = "LD_LIBRARY_PATH";
    let current = std::env::var(KEY).unwrap_or_default();
    if current.split(':').any(|p| !p.is_empty() && p == dir) {
        return;
    }
    let sep = ':';
    std::env::set_var(KEY, format!("{}{}{}", dir, sep, current));
    log::info!("[jlink] Prepended {} to {}", dir, KEY);
}

/// After locating a J-Link install directory, apply PATH (all platforms) and Linux shared-library path.
pub fn ensure_jlink_runtime_env(install_dir: &str) {
    prepend_to_process_path(install_dir);
    #[cfg(target_os = "linux")]
    prepend_ld_library_path_segger_layout(install_dir);
}

/// Linux: prepend `LD_LIBRARY_PATH` entries for a typical SEGGER tree. Some packages put `*.so`
/// under `x86/` or `x86_64/`; search order keeps `install_dir` **first**, then subfolders.
#[cfg(target_os = "linux")]
fn prepend_ld_library_path_segger_layout(install_dir: &str) {
    let base = std::path::Path::new(install_dir);
    let mut extras: Vec<String> = Vec::new();
    for sub in ["x86", "x86_64", "amd64"] {
        let p = base.join(sub);
        if p.is_dir() {
            extras.push(p.to_string_lossy().to_string());
        }
    }
    // Each prepend puts the new segment at the **front**. Prepend subdirs first so `install_dir`
    // ends up leftmost (highest priority).
    for p in extras {
        prepend_ld_library_path(&p);
    }
    prepend_ld_library_path(install_dir);
}
