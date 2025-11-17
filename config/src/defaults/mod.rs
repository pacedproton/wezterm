//! Platform-specific intelligent defaults for WezTerm
//!
//! This module provides smart, platform-aware defaults that make WezTerm
//! work perfectly out of the box without requiring manual configuration.

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(all(unix, not(target_os = "macos")))]
pub mod linux;

use crate::Config;

/// Detect and apply platform-specific intelligent defaults
pub fn apply_smart_defaults(config: &mut Config) {
    #[cfg(target_os = "macos")]
    macos::apply_macos_defaults(config);

    #[cfg(target_os = "windows")]
    windows::apply_windows_defaults(config);

    #[cfg(all(unix, not(target_os = "macos")))]
    linux::apply_linux_defaults(config);
}

/// Check if user has an existing configuration file
pub fn has_user_config() -> bool {
    crate::CONFIG_DIRS
        .iter()
        .any(|dir| dir.join("wezterm.lua").exists() || dir.join("wezterm.toml").exists())
}

/// Get the platform name for diagnostics
pub fn platform_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "macOS"
    }
    #[cfg(target_os = "windows")]
    {
        "Windows"
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        "Linux"
    }
}
