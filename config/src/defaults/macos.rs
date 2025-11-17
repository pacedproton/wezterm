//! macOS-specific intelligent defaults for WezTerm
//!
//! This module detects macOS system settings and provides sensible defaults
//! that make WezTerm feel like a native macOS application.

use crate::{
    AudibleBell, Config, Palette, RgbaColor, TextStyle, VisualBell,
};
use crate::font::{FontAttributes, FontStretch, FontStyle, FontWeight};
use crate::keyassignment::{
    ClipboardCopyDestination, ClipboardPasteSource, KeyAssignment, SpawnCommand, SpawnTabDomain,
};
use crate::keys::{Key, LeaderKey};
use std::time::Duration;
use wezterm_input_types::{Modifiers, WindowDecorations};

/// macOS system appearance detection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemAppearance {
    Light,
    Dark,
    HighContrastLight,
    HighContrastDark,
}

/// macOS accent color
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccentColor {
    Blue,
    Purple,
    Pink,
    Red,
    Orange,
    Yellow,
    Green,
    Graphite,
}

/// macOS font smoothing preference
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontSmoothingLevel {
    Disabled,
    Light,
    Medium,
    Strong,
}

/// Detected macOS system settings
#[derive(Debug, Clone)]
pub struct MacOSSystemSettings {
    pub appearance: SystemAppearance,
    pub accent_color: AccentColor,
    pub font_smoothing: FontSmoothingLevel,
    pub reduced_motion: bool,
    pub high_contrast: bool,
    pub reduce_transparency: bool,
    pub increase_contrast: bool,
}

impl MacOSSystemSettings {
    /// Detect current macOS system settings
    pub fn detect() -> Self {
        Self {
            appearance: Self::detect_appearance(),
            accent_color: Self::detect_accent_color(),
            font_smoothing: Self::detect_font_smoothing(),
            reduced_motion: Self::detect_reduced_motion(),
            high_contrast: Self::detect_high_contrast(),
            reduce_transparency: Self::detect_reduce_transparency(),
            increase_contrast: Self::detect_increase_contrast(),
        }
    }

    fn detect_appearance() -> SystemAppearance {
        #[cfg(target_os = "macos")]
        {
            // Use NSApplication.effectiveAppearance to detect dark mode
            // This is a simplified version - real implementation would use objc runtime
            if Self::is_dark_mode() {
                if Self::detect_high_contrast() {
                    SystemAppearance::HighContrastDark
                } else {
                    SystemAppearance::Dark
                }
            } else {
                if Self::detect_high_contrast() {
                    SystemAppearance::HighContrastLight
                } else {
                    SystemAppearance::Light
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            SystemAppearance::Dark
        }
    }

    #[cfg(target_os = "macos")]
    fn is_dark_mode() -> bool {
        // Check NSUserDefaults for AppleInterfaceStyle
        std::process::Command::new("defaults")
            .args(&["read", "-g", "AppleInterfaceStyle"])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .eq_ignore_ascii_case("dark")
            })
            .unwrap_or(false)
    }

    fn detect_accent_color() -> AccentColor {
        #[cfg(target_os = "macos")]
        {
            // Read from NSUserDefaults AppleAccentColor
            std::process::Command::new("defaults")
                .args(&["read", "-g", "AppleAccentColor"])
                .output()
                .ok()
                .and_then(|output| {
                    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    match value.as_str() {
                        "-1" => Some(AccentColor::Graphite),
                        "0" => Some(AccentColor::Red),
                        "1" => Some(AccentColor::Orange),
                        "2" => Some(AccentColor::Yellow),
                        "3" => Some(AccentColor::Green),
                        "4" | "" => Some(AccentColor::Blue), // Default
                        "5" => Some(AccentColor::Purple),
                        "6" => Some(AccentColor::Pink),
                        _ => Some(AccentColor::Blue),
                    }
                })
                .unwrap_or(AccentColor::Blue)
        }
        #[cfg(not(target_os = "macos"))]
        {
            AccentColor::Blue
        }
    }

    fn detect_font_smoothing() -> FontSmoothingLevel {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("defaults")
                .args(&["read", "-g", "AppleFontSmoothing"])
                .output()
                .ok()
                .and_then(|output| {
                    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    match value.as_str() {
                        "0" => Some(FontSmoothingLevel::Disabled),
                        "1" => Some(FontSmoothingLevel::Light),
                        "2" => Some(FontSmoothingLevel::Medium),
                        "3" => Some(FontSmoothingLevel::Strong),
                        _ => Some(FontSmoothingLevel::Medium),
                    }
                })
                .unwrap_or(FontSmoothingLevel::Medium)
        }
        #[cfg(not(target_os = "macos"))]
        {
            FontSmoothingLevel::Medium
        }
    }

    fn detect_reduced_motion() -> bool {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("defaults")
                .args(&["read", "-g", "com.apple.universalaccess", "reduceMotion"])
                .output()
                .map(|output| {
                    String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .eq_ignore_ascii_case("1")
                })
                .unwrap_or(false)
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn detect_high_contrast() -> bool {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("defaults")
                .args(&["read", "-g", "com.apple.universalaccess", "highContrastEnabled"])
                .output()
                .map(|output| {
                    String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .eq_ignore_ascii_case("1")
                })
                .unwrap_or(false)
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn detect_reduce_transparency() -> bool {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("defaults")
                .args(&["read", "-g", "com.apple.universalaccess", "reduceTransparency"])
                .output()
                .map(|output| {
                    String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .eq_ignore_ascii_case("1")
                })
                .unwrap_or(false)
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn detect_increase_contrast() -> bool {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("defaults")
                .args(&["read", "-g", "com.apple.universalaccess", "increaseContrast"])
                .output()
                .map(|output| {
                    String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .eq_ignore_ascii_case("1")
                })
                .unwrap_or(false)
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }
}

/// Apply macOS-specific intelligent defaults to configuration
pub fn apply_macos_defaults(config: &mut Config) {
    let system = MacOSSystemSettings::detect();

    // Font configuration - prefer SF Mono with Menlo fallback
    if config.font.font.is_empty() {
        config.font = TextStyle {
            font: vec![
                FontAttributes {
                    family: "SF Mono".to_string(),
                    weight: FontWeight::Regular,
                    stretch: FontStretch::Normal,
                    style: FontStyle::Normal,
                    is_fallback: false,
                    is_synthetic: false,
                    harfbuzz_features: None,
                    freetype_load_target: None,
                    freetype_render_target: None,
                    freetype_load_flags: None,
                    scale: None,
                    assume_emoji_presentation: None,
                },
                FontAttributes {
                    family: "Menlo".to_string(),
                    weight: FontWeight::Regular,
                    stretch: FontStretch::Normal,
                    style: FontStyle::Normal,
                    is_fallback: true,
                    is_synthetic: false,
                    harfbuzz_features: None,
                    freetype_load_target: None,
                    freetype_render_target: None,
                    freetype_load_flags: None,
                    scale: None,
                    assume_emoji_presentation: None,
                },
            ],
            foreground: None,
        };
    }

    // Default font size for macOS (13pt is standard)
    if config.font_size == 12.0 {
        // Only override if using the default
        config.font_size = 13.0;
    }

    // Line height optimized for macOS
    if config.line_height == 1.0 {
        config.line_height = 1.1;
    }

    // Window decorations - prefer integrated titlebar
    config.window_decorations = WindowDecorations::INTEGRATED_BUTTONS | WindowDecorations::RESIZE;

    // Native macOS fullscreen
    config.native_macos_fullscreen_mode = true;

    // Window background based on system preferences
    if !system.reduce_transparency {
        config.window_background_opacity = 0.95;
        config.macos_window_background_blur = 10;
    } else {
        config.window_background_opacity = 1.0;
        config.macos_window_background_blur = 0;
    }

    // Tab bar configuration
    config.use_fancy_tab_bar = true;
    config.hide_tab_bar_if_only_one_tab = false;
    config.tab_bar_at_bottom = false;

    // Scrollback lines
    if config.scrollback_lines < 10000 {
        config.scrollback_lines = 10000;
    }

    // Visual bell configuration based on reduced motion preference
    if system.reduced_motion {
        config.visual_bell = VisualBell {
            fade_in_duration_ms: 0,
            fade_out_duration_ms: 0,
            fade_in_function: crate::bell::EasingFunction::Linear,
            fade_out_function: crate::bell::EasingFunction::Linear,
            target: crate::bell::VisualBellTarget::BackgroundColor,
        };
    } else {
        config.visual_bell = VisualBell {
            fade_in_duration_ms: 75,
            fade_out_duration_ms: 150,
            fade_in_function: crate::bell::EasingFunction::EaseIn,
            fade_out_function: crate::bell::EasingFunction::EaseOut,
            target: crate::bell::VisualBellTarget::BackgroundColor,
        };
    }
    config.audible_bell = AudibleBell::SystemBeep;

    // Color scheme based on system appearance
    if config.color_scheme.is_none() {
        config.color_scheme = Some(match system.appearance {
            SystemAppearance::Dark | SystemAppearance::HighContrastDark => {
                "Builtin Solarized Dark".to_string()
            }
            SystemAppearance::Light | SystemAppearance::HighContrastLight => {
                "Builtin Solarized Light".to_string()
            }
        });
    }

    // Cursor configuration
    if config.default_cursor_style == termwiz::surface::CursorShape::Default {
        config.default_cursor_style = termwiz::surface::CursorShape::SteadyBlock;
    }
    config.cursor_blink_ease_in = crate::bell::EasingFunction::EaseIn;
    config.cursor_blink_ease_out = crate::bell::EasingFunction::EaseOut;

    // Updates
    config.check_for_updates = true;
    config.show_update_window = true;

    // Mouse configuration
    config.bypass_mouse_reporting_modifiers = Modifiers::SHIFT;

    // Apply default macOS key bindings if none are configured
    if config.keys.is_empty() {
        config.keys = default_macos_keybindings();
    }
}

/// Standard macOS keyboard shortcuts that feel native
pub fn default_macos_keybindings() -> Vec<Key> {
    use wezterm_input_types::PhysKeyCode;

    vec![
        // Window management
        Key {
            key: wezterm_input_types::KeyCode::Char('n'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::SpawnWindow,
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('t'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::SpawnTab(SpawnTabDomain::DefaultDomain),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('w'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::CloseCurrentTab { confirm: true },
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('q'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::QuitApplication,
        },

        // Tab navigation
        Key {
            key: wezterm_input_types::KeyCode::Char('['),
            mods: Modifiers::SUPER | Modifiers::SHIFT,
            action: KeyAssignment::ActivateTabRelative(-1),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char(']'),
            mods: Modifiers::SUPER | Modifiers::SHIFT,
            action: KeyAssignment::ActivateTabRelative(1),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('1'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ActivateTab(0),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('2'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ActivateTab(1),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('3'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ActivateTab(2),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('4'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ActivateTab(3),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('5'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ActivateTab(4),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('6'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ActivateTab(5),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('7'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ActivateTab(6),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('8'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ActivateTab(7),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('9'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ActivateTab(-1), // Last tab
        },

        // Pane management
        Key {
            key: wezterm_input_types::KeyCode::Char('d'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::SplitHorizontal(SpawnCommand::default()),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('d'),
            mods: Modifiers::SUPER | Modifiers::SHIFT,
            action: KeyAssignment::SplitVertical(SpawnCommand::default()),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('['),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ActivatePaneDirection(
                crate::keyassignment::PaneDirection::Prev,
            ),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char(']'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ActivatePaneDirection(
                crate::keyassignment::PaneDirection::Next,
            ),
        },

        // Copy/Paste
        Key {
            key: wezterm_input_types::KeyCode::Char('c'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::CopyTo(ClipboardCopyDestination::Clipboard),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('v'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::PasteFrom(ClipboardPasteSource::Clipboard),
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('v'),
            mods: Modifiers::SUPER | Modifiers::SHIFT,
            action: KeyAssignment::PasteFrom(ClipboardPasteSource::PrimarySelection),
        },

        // Find
        Key {
            key: wezterm_input_types::KeyCode::Char('f'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::Search(crate::keyassignment::SearchMode::CurrentSelectionOrEmptyString),
        },

        // Font size
        Key {
            key: wezterm_input_types::KeyCode::Char('+'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::IncreaseFontSize,
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('='),
            mods: Modifiers::SUPER,
            action: KeyAssignment::IncreaseFontSize,
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('-'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::DecreaseFontSize,
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('0'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ResetFontSize,
        },

        // Scrollback
        Key {
            key: wezterm_input_types::KeyCode::Char('k'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ClearScrollback(
                crate::keyassignment::ClipboardCopyDestination::Clipboard,
            ),
        },

        // Full screen
        Key {
            key: wezterm_input_types::KeyCode::Char('f'),
            mods: Modifiers::SUPER | Modifiers::CTRL,
            action: KeyAssignment::ToggleFullScreen,
        },
        Key {
            key: wezterm_input_types::KeyCode::Return,
            mods: Modifiers::SUPER | Modifiers::SHIFT,
            action: KeyAssignment::ToggleFullScreen,
        },

        // Window management
        Key {
            key: wezterm_input_types::KeyCode::Char('m'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::Hide,
        },
        Key {
            key: wezterm_input_types::KeyCode::Char('h'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::HideApplication,
        },

        // Quick select
        Key {
            key: wezterm_input_types::KeyCode::Char(' '),
            mods: Modifiers::SUPER | Modifiers::SHIFT,
            action: KeyAssignment::QuickSelect,
        },

        // Debug
        Key {
            key: wezterm_input_types::KeyCode::Char('l'),
            mods: Modifiers::SUPER | Modifiers::SHIFT,
            action: KeyAssignment::ShowDebugOverlay,
        },

        // Reload configuration
        Key {
            key: wezterm_input_types::KeyCode::Char('r'),
            mods: Modifiers::SUPER,
            action: KeyAssignment::ReloadConfiguration,
        },
    ]
}

/// Get accent color as RGBA
pub fn accent_color_to_rgba(accent: AccentColor) -> RgbaColor {
    match accent {
        AccentColor::Blue => RgbaColor::new_8bpc(0, 122, 255, 255),
        AccentColor::Purple => RgbaColor::new_8bpc(175, 82, 222, 255),
        AccentColor::Pink => RgbaColor::new_8bpc(255, 45, 85, 255),
        AccentColor::Red => RgbaColor::new_8bpc(255, 59, 48, 255),
        AccentColor::Orange => RgbaColor::new_8bpc(255, 149, 0, 255),
        AccentColor::Yellow => RgbaColor::new_8bpc(255, 204, 0, 255),
        AccentColor::Green => RgbaColor::new_8bpc(52, 199, 89, 255),
        AccentColor::Graphite => RgbaColor::new_8bpc(142, 142, 147, 255),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_settings_detection() {
        let settings = MacOSSystemSettings::detect();
        // Should not panic
        assert!(matches!(
            settings.appearance,
            SystemAppearance::Light
                | SystemAppearance::Dark
                | SystemAppearance::HighContrastLight
                | SystemAppearance::HighContrastDark
        ));
    }

    #[test]
    fn test_default_keybindings_not_empty() {
        let keys = default_macos_keybindings();
        assert!(!keys.is_empty());
        // Should have common shortcuts
        assert!(keys.iter().any(|k| k.action == KeyAssignment::SpawnWindow));
        assert!(keys.iter().any(|k| k.action == KeyAssignment::QuitApplication));
    }

    #[test]
    fn test_accent_color_to_rgba() {
        let blue = accent_color_to_rgba(AccentColor::Blue);
        assert_eq!(blue.0, 0);
        assert_eq!(blue.1, 122);
        assert_eq!(blue.2, 255);
        assert_eq!(blue.3, 255);
    }
}
