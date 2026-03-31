//! Linux-specific search directories for J-Link.

use std::path::PathBuf;

pub fn search_dirs() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/usr/bin"),
        PathBuf::from("/usr/local/bin"),
        // Standard SEGGER installers use /opt/SEGGER/JLink
        PathBuf::from("/opt/SEGGER/JLink"),
        // Lite bundles extract into /opt/SEGGER/JLink_V930a
        PathBuf::from("/opt/SEGGER"),
    ]
}