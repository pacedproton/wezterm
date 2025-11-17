/*
 * WezTermBridge - Swift wrapper for WezTerm Rust FFI
 *
 * This module provides a Swift-friendly interface to the WezTerm Rust core.
 */

import Foundation

/// Main bridge to WezTerm Rust core
public final class WezTermBridge {
    /// Shared singleton instance
    public static let shared = WezTermBridge()

    private init() {}

    // MARK: - Configuration

    /// Get current configuration
    public func getCurrentConfig() -> WezTermConfigWrapper {
        let config = wezterm_config_new()
        return WezTermConfigWrapper(config: config!)
    }

    /// Apply configuration changes
    public func applyConfig(_ config: WezTermConfigWrapper) -> Bool {
        return wezterm_config_apply(config.rawPointer)
    }

    /// Load configuration from file
    public func loadConfig(from path: String) -> WezTermConfigWrapper? {
        guard let config = wezterm_config_load_from_file(path) else {
            return nil
        }
        return WezTermConfigWrapper(config: config)
    }

    /// Save configuration to file
    public func saveConfig(_ config: WezTermConfigWrapper, to path: String) -> Bool {
        return wezterm_config_save_to_file(config.rawPointer, path)
    }

    // MARK: - System Detection

    /// Detect system appearance
    public func detectAppearance() -> SystemAppearance {
        guard let cStr = wezterm_detect_appearance() else {
            return .dark
        }
        let str = String(cString: cStr)
        switch str {
        case "light":
            return .light
        case "dark":
            return .dark
        case "high_contrast_light":
            return .highContrastLight
        case "high_contrast_dark":
            return .highContrastDark
        default:
            return .dark
        }
    }

    /// Detect system accent color
    public func detectAccentColor() -> AccentColor {
        guard let cStr = wezterm_detect_accent_color() else {
            return .blue
        }
        let str = String(cString: cStr)
        return AccentColor(rawValue: str) ?? .blue
    }

    /// Check if reduced motion is enabled
    public func isReducedMotionEnabled() -> Bool {
        return wezterm_detect_reduced_motion()
    }

    /// Check if reduce transparency is enabled
    public func isReduceTransparencyEnabled() -> Bool {
        return wezterm_detect_reduce_transparency()
    }

    /// Check if high contrast is enabled
    public func isHighContrastEnabled() -> Bool {
        return wezterm_detect_high_contrast()
    }

    /// Get font smoothing level
    public func getFontSmoothingLevel() -> Int {
        return Int(wezterm_detect_font_smoothing())
    }

    // MARK: - Color Schemes

    /// Get available color schemes
    public func getAvailableColorSchemes() -> [String] {
        var count: Int = 0
        guard let schemes = wezterm_get_available_color_schemes(&count) else {
            return []
        }

        var result: [String] = []
        for i in 0..<count {
            if let cStr = schemes[i] {
                result.append(String(cString: cStr))
            }
        }

        wezterm_free_color_scheme_list(schemes, count)
        return result
    }

    /// Get color scheme by name
    public func getColorScheme(named name: String) -> ColorScheme? {
        guard let scheme = wezterm_get_color_scheme(name) else {
            return nil
        }

        let fg = wezterm_color_scheme_get_foreground(scheme)
        let bg = wezterm_color_scheme_get_background(scheme)

        var colors: [Color] = []
        for i: UInt8 in 0..<16 {
            let c = wezterm_color_scheme_get_ansi(scheme, i)
            colors.append(Color(r: c.r, g: c.g, b: c.b, a: c.a))
        }

        wezterm_color_scheme_free(scheme)

        return ColorScheme(
            name: name,
            foreground: Color(r: fg.r, g: fg.g, b: fg.b, a: fg.a),
            background: Color(r: bg.r, g: bg.g, b: bg.b, a: bg.a),
            ansiColors: colors
        )
    }

    // MARK: - Application Control

    /// Open a new window
    public func openNewWindow() {
        wezterm_open_new_window()
    }

    /// Open a new tab
    public func openNewTab() {
        wezterm_open_new_tab()
    }

    /// Open a new tab with specific path
    public func openNewTabWithPath(_ path: String) {
        wezterm_open_new_tab_with_path(path)
    }

    /// Reload configuration
    public func reloadConfiguration() {
        wezterm_reload_configuration()
    }

    /// Show preferences window
    public func showPreferences() {
        wezterm_show_preferences()
    }

    /// Get application version
    public func getVersion() -> String {
        guard let cStr = wezterm_get_version() else {
            return "Unknown"
        }
        return String(cString: cStr)
    }
}

// MARK: - Configuration Wrapper

/// Swift wrapper for WezTerm configuration
public final class WezTermConfigWrapper {
    internal let rawPointer: UnsafeMutablePointer<WezTermConfig>

    internal init(config: UnsafeMutablePointer<WezTermConfig>) {
        self.rawPointer = config
    }

    deinit {
        wezterm_config_free(rawPointer)
    }

    // MARK: - Font Settings

    public var fontFamily: String {
        get {
            guard let cStr = wezterm_config_get_font_family(rawPointer) else {
                return "SF Mono"
            }
            return String(cString: cStr)
        }
        set {
            wezterm_config_set_font_family(rawPointer, newValue)
        }
    }

    public var fontSize: Double {
        get { wezterm_config_get_font_size(rawPointer) }
        set { wezterm_config_set_font_size(rawPointer, newValue) }
    }

    public var lineHeight: Double {
        get { wezterm_config_get_line_height(rawPointer) }
        set { wezterm_config_set_line_height(rawPointer, newValue) }
    }

    public var fontWeight: UInt16 {
        get { wezterm_config_get_font_weight(rawPointer) }
        set { wezterm_config_set_font_weight(rawPointer, newValue) }
    }

    // MARK: - Color Settings

    public var colorScheme: String {
        get {
            guard let cStr = wezterm_config_get_color_scheme(rawPointer) else {
                return ""
            }
            return String(cString: cStr)
        }
        set {
            wezterm_config_set_color_scheme(rawPointer, newValue)
        }
    }

    public var foregroundColor: Color {
        get {
            let c = wezterm_config_get_foreground_color(rawPointer)
            return Color(r: c.r, g: c.g, b: c.b, a: c.a)
        }
        set {
            let c = WezTermColor(r: newValue.r, g: newValue.g, b: newValue.b, a: newValue.a)
            wezterm_config_set_foreground_color(rawPointer, c)
        }
    }

    public var backgroundColor: Color {
        get {
            let c = wezterm_config_get_background_color(rawPointer)
            return Color(r: c.r, g: c.g, b: c.b, a: c.a)
        }
        set {
            let c = WezTermColor(r: newValue.r, g: newValue.g, b: newValue.b, a: newValue.a)
            wezterm_config_set_background_color(rawPointer, c)
        }
    }

    // MARK: - Window Settings

    public var windowOpacity: Double {
        get { wezterm_config_get_window_opacity(rawPointer) }
        set { wezterm_config_set_window_opacity(rawPointer, newValue) }
    }

    public var backgroundBlur: Int32 {
        get { wezterm_config_get_background_blur(rawPointer) }
        set { wezterm_config_set_background_blur(rawPointer, newValue) }
    }

    public var initialRows: UInt16 {
        get { wezterm_config_get_initial_rows(rawPointer) }
        set { wezterm_config_set_initial_rows(rawPointer, newValue) }
    }

    public var initialCols: UInt16 {
        get { wezterm_config_get_initial_cols(rawPointer) }
        set { wezterm_config_set_initial_cols(rawPointer, newValue) }
    }

    public var scrollbackLines: Int {
        get { Int(wezterm_config_get_scrollback_lines(rawPointer)) }
        set { wezterm_config_set_scrollback_lines(rawPointer, newValue) }
    }

    // MARK: - Cursor Settings

    public var cursorStyle: CursorStyle {
        get {
            let style = wezterm_config_get_cursor_style(rawPointer)
            switch style {
            case WEZTERM_CURSOR_BLOCK:
                return .block
            case WEZTERM_CURSOR_UNDERLINE:
                return .underline
            case WEZTERM_CURSOR_BAR:
                return .bar
            default:
                return .block
            }
        }
        set {
            let style: WezTermCursorStyle
            switch newValue {
            case .block:
                style = WEZTERM_CURSOR_BLOCK
            case .underline:
                style = WEZTERM_CURSOR_UNDERLINE
            case .bar:
                style = WEZTERM_CURSOR_BAR
            }
            wezterm_config_set_cursor_style(rawPointer, style)
        }
    }

    public var cursorBlinks: Bool {
        get { wezterm_config_get_cursor_blink(rawPointer) }
        set { wezterm_config_set_cursor_blink(rawPointer, newValue) }
    }

    // MARK: - Tab Bar Settings

    public var hideTabBarIfOnlyOneTab: Bool {
        get { wezterm_config_get_hide_tab_bar_if_only_one_tab(rawPointer) }
        set { wezterm_config_set_hide_tab_bar_if_only_one_tab(rawPointer, newValue) }
    }

    public var tabBarAtBottom: Bool {
        get { wezterm_config_get_tab_bar_at_bottom(rawPointer) }
        set { wezterm_config_set_tab_bar_at_bottom(rawPointer, newValue) }
    }

    public var useFancyTabBar: Bool {
        get { wezterm_config_get_use_fancy_tab_bar(rawPointer) }
        set { wezterm_config_set_use_fancy_tab_bar(rawPointer, newValue) }
    }

    // MARK: - Behavior Settings

    public var checkForUpdates: Bool {
        get { wezterm_config_get_check_for_updates(rawPointer) }
        set { wezterm_config_set_check_for_updates(rawPointer, newValue) }
    }

    public var nativeFullscreen: Bool {
        get { wezterm_config_get_native_fullscreen(rawPointer) }
        set { wezterm_config_set_native_fullscreen(rawPointer, newValue) }
    }

    public var confirmCloseProcess: Bool {
        get { wezterm_config_get_confirm_close_process(rawPointer) }
        set { wezterm_config_set_confirm_close_process(rawPointer, newValue) }
    }
}

// MARK: - Supporting Types

/// System appearance
public enum SystemAppearance: String {
    case light
    case dark
    case highContrastLight = "high_contrast_light"
    case highContrastDark = "high_contrast_dark"
}

/// System accent color
public enum AccentColor: String {
    case blue
    case purple
    case pink
    case red
    case orange
    case yellow
    case green
    case graphite
}

/// Cursor style
public enum CursorStyle: String, CaseIterable {
    case block
    case underline
    case bar
}

/// RGB color
public struct Color: Equatable {
    public let r: UInt8
    public let g: UInt8
    public let b: UInt8
    public let a: UInt8

    public init(r: UInt8, g: UInt8, b: UInt8, a: UInt8 = 255) {
        self.r = r
        self.g = g
        self.b = b
        self.a = a
    }

    public init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet.alphanumerics.inverted)
        var int: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&int)
        let r, g, b, a: UInt64
        switch hex.count {
        case 3: // RGB (12-bit)
            (r, g, b, a) = ((int >> 8) * 17, (int >> 4 & 0xF) * 17, (int & 0xF) * 17, 255)
        case 6: // RGB (24-bit)
            (r, g, b, a) = (int >> 16, int >> 8 & 0xFF, int & 0xFF, 255)
        case 8: // ARGB (32-bit)
            (r, g, b, a) = (int >> 24, int >> 16 & 0xFF, int >> 8 & 0xFF, int & 0xFF)
        default:
            (r, g, b, a) = (0, 0, 0, 255)
        }
        self.r = UInt8(r)
        self.g = UInt8(g)
        self.b = UInt8(b)
        self.a = UInt8(a)
    }

    public var hexString: String {
        return String(format: "#%02X%02X%02X", r, g, b)
    }
}

/// Color scheme
public struct ColorScheme {
    public let name: String
    public let foreground: Color
    public let background: Color
    public let ansiColors: [Color]
}
