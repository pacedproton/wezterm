# WezTerm macOS Native Experience & LibVT Library Extraction Specification

**Version:** 1.0
**Date:** 2025-11-17
**Status:** Draft

---

## Executive Summary

This specification document outlines two major initiatives for WezTerm:

1. **macOS Native Experience Enhancement** - Transform WezTerm into a first-class macOS citizen with native UI/UX, Metal rendering, and zero-configuration usability
2. **LibVT Library Extraction** - Extract the terminal emulation core into a standalone library for integration with VSCode, editors, and other applications

Both initiatives are complementary and will result in a more maintainable, performant, and widely-adopted terminal emulation stack.

---

## Table of Contents

1. [Initiative A: macOS Native Experience](#initiative-a-macos-native-experience)
   - [Goals & Non-Goals](#a1-goals--non-goals)
   - [Architecture Overview](#a2-architecture-overview)
   - [Design Specifications](#a3-design-specifications)
   - [Implementation Plan](#a4-implementation-plan)
   - [Testing Strategy](#a5-testing-strategy)

2. [Initiative B: LibVT Library Extraction](#initiative-b-libvt-library-extraction)
   - [Goals & Non-Goals](#b1-goals--non-goals)
   - [Architecture Overview](#b2-architecture-overview)
   - [API Design](#b3-api-design)
   - [Implementation Plan](#b4-implementation-plan)
   - [Testing Strategy](#b5-testing-strategy)

3. [Shared Infrastructure](#shared-infrastructure)
4. [Migration Strategy](#migration-strategy)
5. [Risk Assessment](#risk-assessment)
6. [Success Metrics](#success-metrics)

---

## Initiative A: macOS Native Experience

### A1. Goals & Non-Goals

#### Goals

1. **Zero-Configuration Usability**
   - Works perfectly out of the box
   - Sensible defaults for macOS users
   - No Lua configuration required for basic usage
   - GUI preferences panel for common settings

2. **Native macOS UI/UX**
   - Native macOS menu bar with standard shortcuts
   - Native file dialogs (Open, Save)
   - Native color picker integration
   - System Preferences integration
   - Spotlight and Finder integration
   - Dock badge notifications
   - Touch Bar support

3. **Modern Graphics Performance**
   - Metal rendering backend (replacing OpenGL 3.2)
   - ProMotion support (120Hz on supported displays)
   - Efficient memory usage
   - GPU-accelerated text rendering

4. **System Integration**
   - Notification Center integration
   - iCloud sync for settings
   - Handoff support between devices
   - Siri Shortcuts integration
   - Screen Time integration

5. **Accessibility**
   - Full VoiceOver support
   - Accessibility Inspector compliance
   - High contrast mode support
   - Reduced motion support
   - Voice Control compatibility

#### Non-Goals

- Breaking existing Lua configuration support
- Removing advanced configuration options
- Windows/Linux parity for native features
- Complete UI rewrite in SwiftUI

---

### A2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    macOS Native WezTerm                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐ │
│  │   SwiftUI    │  │   AppKit     │  │   Metal Rendering    │ │
│  │  Preferences │  │   Menus      │  │      Backend         │ │
│  └──────────────┘  └──────────────┘  └──────────────────────┘ │
│         │                 │                      │             │
│         └─────────────────┼──────────────────────┘             │
│                           │                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │               macOS Platform Bridge (Swift)              │   │
│  │  - Objective-C/Swift interop                            │   │
│  │  - Native API wrappers                                  │   │
│  │  - System service integration                           │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           │                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Rust Core (existing)                   │   │
│  │  - Terminal emulation                                   │   │
│  │  - Session management                                   │   │
│  │  - Configuration                                        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### Layer Responsibilities

1. **SwiftUI/AppKit Native Layer**
   - Preferences UI
   - About/Welcome windows
   - Native dialogs
   - Touch Bar interface

2. **macOS Platform Bridge**
   - Swift/Rust FFI
   - Metal rendering context
   - System service integration
   - Accessibility bridge

3. **Enhanced Rust Core**
   - Metal rendering backend
   - Smart defaults system
   - Simplified configuration API
   - Performance optimizations

---

### A3. Design Specifications

#### A3.1 Default Configuration System

**New Module:** `config/src/defaults/macos.rs`

```rust
/// macOS-specific intelligent defaults
pub struct MacOSDefaults {
    /// Automatically detected settings
    pub appearance: SystemAppearance,
    pub accent_color: AccentColor,
    pub font_smoothing: FontSmoothingLevel,
    pub reduced_motion: bool,
    pub high_contrast: bool,
}

impl MacOSDefaults {
    pub fn detect() -> Self {
        MacOSDefaults {
            appearance: Self::detect_appearance(),
            accent_color: Self::detect_accent_color(),
            font_smoothing: Self::detect_font_smoothing(),
            reduced_motion: Self::detect_reduced_motion(),
            high_contrast: Self::detect_high_contrast(),
        }
    }
}

/// Default configuration for macOS (no user config required)
pub fn default_macos_config() -> Config {
    let system = MacOSDefaults::detect();

    Config {
        // Use system font with fallbacks
        font: TextStyle {
            font: vec![
                FontAttributes {
                    family: "SF Mono".to_string(),
                    weight: FontWeight::Regular,
                    stretch: FontStretch::Normal,
                    style: FontStyle::Normal,
                },
                FontAttributes {
                    family: "Menlo".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        },

        // Sensible defaults
        font_size: 13.0,  // macOS standard
        line_height: 1.1,

        // Colors follow system appearance
        color_scheme: match system.appearance {
            SystemAppearance::Dark => "Builtin Solarized Dark".to_string(),
            SystemAppearance::Light => "Builtin Solarized Light".to_string(),
        },

        // macOS-specific window behavior
        window_decorations: WindowDecorations::Integrated,
        native_macos_fullscreen_mode: true,
        window_background_opacity: 0.95,
        macos_window_background_blur: 10,

        // Tab bar
        tab_bar_at_bottom: false,
        use_fancy_tab_bar: true,
        hide_tab_bar_if_only_one_tab: false,

        // Scrollback
        scrollback_lines: 10000,

        // Bell
        visual_bell: VisualBellConfig {
            fade_in_duration_ms: 75,
            fade_out_duration_ms: 150,
            ..Default::default()
        },
        audible_bell: AudibleBell::SystemBeep,

        // Standard macOS key bindings
        keys: default_macos_keybindings(),

        // Mouse
        bypass_mouse_reporting_modifiers: Modifiers::SHIFT,

        // Updates
        check_for_updates: true,
        show_update_window: true,

        ..Default::default()
    }
}

/// Standard macOS keyboard shortcuts
fn default_macos_keybindings() -> Vec<KeyBinding> {
    vec![
        // Standard macOS shortcuts
        key!(CMD, "N", SpawnWindow(SpawnCommand::default())),
        key!(CMD, "T", SpawnTab(SpawnTabDomain::DefaultDomain)),
        key!(CMD, "W", CloseCurrentTab { confirm: true }),
        key!(CMD, "Q", QuitApplication),

        // Tabs
        key!(CMD | SHIFT, "[", ActivateTabRelative(-1)),
        key!(CMD | SHIFT, "]", ActivateTabRelative(1)),
        key!(CMD, "1", ActivateTab(0)),
        key!(CMD, "2", ActivateTab(1)),
        // ... 1-9

        // Panes
        key!(CMD, "D", SplitHorizontal(SpawnCommand::default())),
        key!(CMD | SHIFT, "D", SplitVertical(SpawnCommand::default())),
        key!(CMD, "[", ActivatePaneDirection(PaneDirection::Prev)),
        key!(CMD, "]", ActivatePaneDirection(PaneDirection::Next)),

        // Copy/Paste
        key!(CMD, "C", CopyTo(ClipboardCopyDestination::Clipboard)),
        key!(CMD, "V", PasteFrom(ClipboardPasteSource::Clipboard)),
        key!(CMD | SHIFT, "V", PasteFrom(ClipboardPasteSource::PrimarySelection)),

        // Find
        key!(CMD, "F", Search(Pattern::default())),
        key!(CMD, "G", FindNext),
        key!(CMD | SHIFT, "G", FindPrev),

        // Font size
        key!(CMD, "+", IncreaseFontSize),
        key!(CMD, "-", DecreaseFontSize),
        key!(CMD, "0", ResetFontSize),

        // Scrolling
        key!(CMD, "K", ClearScrollback(ClearScrollback::ScrollbackOnly)),
        key!(CMD | SHIFT, "K", ClearScrollback(ClearScrollback::ScrollbackAndViewport)),

        // Full screen
        key!(CMD | CTRL, "F", ToggleFullScreen),
        key!(CMD | SHIFT, "ENTER", ToggleFullScreen),

        // Window management
        key!(CMD, "M", Hide),
        key!(CMD, "H", HideApplication),
        key!(CMD | ALT, "H", HideOtherApplications),

        // Quick select
        key!(CMD | SHIFT, "Space", QuickSelect),
    ]
}
```

#### A3.2 Metal Rendering Backend

**New Module:** `window/src/os/macos/metal.rs`

```rust
use metal::{Device, CommandQueue, RenderPipelineState};

/// Metal rendering context for macOS
pub struct MetalRenderContext {
    device: Device,
    command_queue: CommandQueue,
    pipeline_state: RenderPipelineState,
    glyph_atlas: MetalTextureAtlas,
    vertex_buffer: MetalBuffer,
}

impl MetalRenderContext {
    pub fn new() -> Result<Self> {
        let device = Device::system_default()
            .ok_or_else(|| anyhow!("No Metal device found"))?;

        let command_queue = device.new_command_queue();

        // Create shader library
        let library = device.new_library_with_source(METAL_SHADERS, &metal::CompileOptions::new())?;

        // Create pipeline
        let vertex_func = library.get_function("vertex_main", None)?;
        let fragment_func = library.get_function("fragment_main", None)?;

        let pipeline_desc = metal::RenderPipelineDescriptor::new();
        pipeline_desc.set_vertex_function(Some(&vertex_func));
        pipeline_desc.set_fragment_function(Some(&fragment_func));

        let pipeline_state = device.new_render_pipeline_state(&pipeline_desc)?;

        Ok(Self {
            device,
            command_queue,
            pipeline_state,
            glyph_atlas: MetalTextureAtlas::new(&device)?,
            vertex_buffer: MetalBuffer::new(&device)?,
        })
    }

    pub fn render_frame(&mut self, terminal_state: &TerminalState) -> Result<()> {
        let command_buffer = self.command_queue.new_command_buffer();

        // Build render pass
        let render_pass = self.build_render_pass()?;
        let encoder = command_buffer.new_render_command_encoder(&render_pass);

        encoder.set_render_pipeline_state(&self.pipeline_state);

        // Render terminal lines
        for line in terminal_state.visible_lines() {
            self.render_line(&encoder, line)?;
        }

        encoder.end_encoding();
        command_buffer.present_drawable(&self.drawable);
        command_buffer.commit();

        Ok(())
    }
}

const METAL_SHADERS: &str = r#"
#include <metal_stdlib>
using namespace metal;

struct VertexIn {
    float2 position [[attribute(0)]];
    float2 texcoord [[attribute(1)]];
    float4 fg_color [[attribute(2)]];
    float4 bg_color [[attribute(3)]];
};

struct VertexOut {
    float4 position [[position]];
    float2 texcoord;
    float4 fg_color;
    float4 bg_color;
};

vertex VertexOut vertex_main(VertexIn in [[stage_in]]) {
    VertexOut out;
    out.position = float4(in.position, 0.0, 1.0);
    out.texcoord = in.texcoord;
    out.fg_color = in.fg_color;
    out.bg_color = in.bg_color;
    return out;
}

fragment float4 fragment_main(
    VertexOut in [[stage_in]],
    texture2d<float> glyph_atlas [[texture(0)]]
) {
    constexpr sampler s(mag_filter::linear, min_filter::linear);
    float4 glyph = glyph_atlas.sample(s, in.texcoord);
    return mix(in.bg_color, in.fg_color, glyph.r);
}
"#;
```

#### A3.3 SwiftUI Preferences Panel

**New File:** `macos-native/Sources/Preferences/PreferencesView.swift`

```swift
import SwiftUI
import WezTermBridge

struct PreferencesView: View {
    @StateObject private var viewModel = PreferencesViewModel()

    var body: some View {
        TabView {
            GeneralPreferencesView(viewModel: viewModel)
                .tabItem {
                    Label("General", systemImage: "gear")
                }

            AppearancePreferencesView(viewModel: viewModel)
                .tabItem {
                    Label("Appearance", systemImage: "paintbrush")
                }

            FontPreferencesView(viewModel: viewModel)
                .tabItem {
                    Label("Fonts", systemImage: "textformat")
                }

            KeyboardPreferencesView(viewModel: viewModel)
                .tabItem {
                    Label("Keyboard", systemImage: "keyboard")
                }

            ShellPreferencesView(viewModel: viewModel)
                .tabItem {
                    Label("Shell", systemImage: "terminal")
                }

            AdvancedPreferencesView(viewModel: viewModel)
                .tabItem {
                    Label("Advanced", systemImage: "gearshape.2")
                }
        }
        .frame(width: 600, height: 400)
    }
}

struct GeneralPreferencesView: View {
    @ObservedObject var viewModel: PreferencesViewModel

    var body: some View {
        Form {
            Section("Startup") {
                Picker("New window opens with:", selection: $viewModel.newWindowBehavior) {
                    Text("Default Profile").tag(NewWindowBehavior.defaultProfile)
                    Text("Same Working Directory").tag(NewWindowBehavior.sameDirectory)
                }

                Toggle("Open new window on launch", isOn: $viewModel.openWindowOnLaunch)
                Toggle("Restore windows when re-opening app", isOn: $viewModel.restoreWindows)
            }

            Section("Tabs") {
                Toggle("Show tab bar", isOn: $viewModel.showTabBar)
                Toggle("Hide tab bar when only one tab", isOn: $viewModel.hideTabBarIfOnlyOneTab)
                Picker("Tab bar position:", selection: $viewModel.tabBarPosition) {
                    Text("Top").tag(TabBarPosition.top)
                    Text("Bottom").tag(TabBarPosition.bottom)
                }
            }

            Section("Window") {
                Toggle("Native macOS full screen", isOn: $viewModel.nativeFullScreen)
                Toggle("Confirm on quit", isOn: $viewModel.confirmOnQuit)
            }

            Section("Updates") {
                Toggle("Check for updates automatically", isOn: $viewModel.checkForUpdates)
            }
        }
        .padding()
    }
}

struct AppearancePreferencesView: View {
    @ObservedObject var viewModel: PreferencesViewModel

    var body: some View {
        Form {
            Section("Theme") {
                Picker("Color Scheme:", selection: $viewModel.colorScheme) {
                    ForEach(viewModel.availableColorSchemes, id: \.self) { scheme in
                        Text(scheme)
                    }
                }

                Toggle("Adjust for system appearance", isOn: $viewModel.adjustForSystemAppearance)
            }

            Section("Window") {
                Slider(value: $viewModel.windowOpacity, in: 0.5...1.0) {
                    Text("Window Opacity:")
                }

                Slider(value: $viewModel.backgroundBlur, in: 0...30) {
                    Text("Background Blur:")
                }
            }

            Section("Cursor") {
                Picker("Cursor style:", selection: $viewModel.cursorStyle) {
                    Text("Block").tag(CursorStyle.block)
                    Text("Bar").tag(CursorStyle.bar)
                    Text("Underline").tag(CursorStyle.underline)
                }

                Toggle("Cursor blinks", isOn: $viewModel.cursorBlinks)
            }
        }
        .padding()
    }
}

struct FontPreferencesView: View {
    @ObservedObject var viewModel: PreferencesViewModel

    var body: some View {
        Form {
            Section("Font") {
                HStack {
                    Text("Font:")
                    Spacer()
                    Button("\(viewModel.fontFamily) \(Int(viewModel.fontSize))") {
                        viewModel.showFontPicker = true
                    }
                }
                .sheet(isPresented: $viewModel.showFontPicker) {
                    FontPickerView(selectedFont: $viewModel.fontFamily,
                                  fontSize: $viewModel.fontSize)
                }

                Slider(value: $viewModel.lineHeight, in: 0.8...2.0) {
                    Text("Line Height:")
                }
            }

            Section("Rendering") {
                Picker("Font smoothing:", selection: $viewModel.fontSmoothing) {
                    Text("System Default").tag(FontSmoothing.system)
                    Text("Light").tag(FontSmoothing.light)
                    Text("Medium").tag(FontSmoothing.medium)
                    Text("Strong").tag(FontSmoothing.strong)
                }

                Toggle("Use ligatures", isOn: $viewModel.useLigatures)
            }
        }
        .padding()
    }
}

@MainActor
class PreferencesViewModel: ObservableObject {
    @Published var newWindowBehavior: NewWindowBehavior = .defaultProfile
    @Published var openWindowOnLaunch: Bool = true
    @Published var restoreWindows: Bool = true
    @Published var showTabBar: Bool = true
    @Published var hideTabBarIfOnlyOneTab: Bool = false
    @Published var tabBarPosition: TabBarPosition = .top
    @Published var nativeFullScreen: Bool = true
    @Published var confirmOnQuit: Bool = true
    @Published var checkForUpdates: Bool = true

    @Published var colorScheme: String = "Solarized Dark"
    @Published var adjustForSystemAppearance: Bool = true
    @Published var windowOpacity: Double = 0.95
    @Published var backgroundBlur: Double = 10
    @Published var cursorStyle: CursorStyle = .block
    @Published var cursorBlinks: Bool = true

    @Published var fontFamily: String = "SF Mono"
    @Published var fontSize: Double = 13
    @Published var lineHeight: Double = 1.1
    @Published var fontSmoothing: FontSmoothing = .system
    @Published var useLigatures: Bool = false

    @Published var showFontPicker: Bool = false

    var availableColorSchemes: [String] {
        WezTermBridge.shared.getAvailableColorSchemes()
    }

    init() {
        loadFromConfig()
    }

    private func loadFromConfig() {
        let config = WezTermBridge.shared.getCurrentConfig()
        // Load values from Rust config...
    }

    func save() {
        WezTermBridge.shared.applyConfig(self.toRustConfig())
    }
}
```

#### A3.4 System Service Integration

**New Module:** `macos-native/Sources/Services/SystemServices.swift`

```swift
import Foundation
import UserNotifications
import Intents
import AppKit

/// Integration with macOS system services
class SystemServices {
    static let shared = SystemServices()

    // MARK: - Notification Center

    func requestNotificationPermission() async -> Bool {
        let center = UNUserNotificationCenter.current()
        do {
            let granted = try await center.requestAuthorization(options: [.alert, .sound, .badge])
            return granted
        } catch {
            return false
        }
    }

    func showNotification(title: String, body: String, identifier: String) {
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        content.sound = .default

        let request = UNNotificationRequest(
            identifier: identifier,
            content: content,
            trigger: nil
        )

        UNUserNotificationCenter.current().add(request)
    }

    // MARK: - Dock Badge

    func updateDockBadge(count: Int) {
        NSApplication.shared.dockTile.badgeLabel = count > 0 ? "\(count)" : nil
    }

    func clearDockBadge() {
        NSApplication.shared.dockTile.badgeLabel = nil
    }

    // MARK: - Spotlight Integration

    func indexTerminalProfile(_ profile: TerminalProfile) {
        let attributeSet = CSSearchableItemAttributeSet(contentType: .item)
        attributeSet.title = profile.name
        attributeSet.contentDescription = "WezTerm Profile: \(profile.description)"

        let item = CSSearchableItem(
            uniqueIdentifier: "profile-\(profile.id)",
            domainIdentifier: "com.wezterm.profiles",
            attributeSet: attributeSet
        )

        CSSearchableIndex.default().indexSearchableItems([item])
    }

    // MARK: - Siri Shortcuts

    func donateOpenTerminalIntent() {
        let intent = OpenTerminalIntent()
        intent.suggestedInvocationPhrase = "Open Terminal"

        let interaction = INInteraction(intent: intent, response: nil)
        interaction.donate { error in
            if let error = error {
                print("Failed to donate intent: \(error)")
            }
        }
    }

    // MARK: - Handoff

    func startHandoffActivity(workingDirectory: String) -> NSUserActivity {
        let activity = NSUserActivity(activityType: "com.wezterm.terminal")
        activity.title = "WezTerm Session"
        activity.userInfo = ["workingDirectory": workingDirectory]
        activity.isEligibleForHandoff = true
        activity.becomeCurrent()
        return activity
    }

    // MARK: - Services Menu

    func registerServices() {
        NSApplication.shared.servicesProvider = TerminalServicesProvider.shared
    }
}

class TerminalServicesProvider: NSObject {
    static let shared = TerminalServicesProvider()

    @objc func openInWezTerm(_ pboard: NSPasteboard, userData: String, error: AutoreleasingUnsafeMutablePointer<NSString>) {
        guard let path = pboard.string(forType: .fileURL) else { return }
        WezTermBridge.shared.openNewTabWithPath(path)
    }
}
```

#### A3.5 Accessibility Implementation

**New Module:** `window/src/os/macos/accessibility.rs`

```rust
use cocoa::appkit::NSAccessibilityProtocol;
use objc::{class, msg_send, sel, sel_impl};

/// Accessibility support for VoiceOver and other assistive technologies
pub struct AccessibilityBridge {
    terminal_view: id,
}

impl AccessibilityBridge {
    pub fn new(view: id) -> Self {
        Self { terminal_view: view }
    }

    /// Announce text change to screen reader
    pub fn announce_output(&self, text: &str) {
        unsafe {
            let ns_string: id = NSString::alloc(nil).init_str(text);
            let notification_name = NSString::alloc(nil)
                .init_str("NSAccessibilityAnnouncementRequestedNotification");

            NSAccessibilityPostNotification(
                self.terminal_view,
                notification_name,
            );
        }
    }

    /// Update cursor position for screen readers
    pub fn update_cursor_position(&self, row: usize, col: usize) {
        unsafe {
            let user_info: id = msg_send![class!(NSDictionary), dictionaryWithObjectsAndKeys:
                NSNumber::numberWithUnsignedInteger_(nil, row as u64),
                NSString::alloc(nil).init_str("row"),
                NSNumber::numberWithUnsignedInteger_(nil, col as u64),
                NSString::alloc(nil).init_str("column"),
                nil
            ];

            NSAccessibilityPostNotificationWithUserInfo(
                self.terminal_view,
                NSString::alloc(nil).init_str("NSAccessibilitySelectedTextChangedNotification"),
                user_info,
            );
        }
    }

    /// Get accessible description for current line
    pub fn get_line_description(&self, line: &Line) -> String {
        let mut description = String::new();

        for cell in line.cells() {
            description.push_str(&cell.str());
        }

        // Announce special attributes
        if line.has_hyperlinks() {
            description.push_str(" (contains links)");
        }

        description
    }
}

/// Custom NSView methods for accessibility
#[allow(non_snake_case)]
mod accessibility_methods {
    use super::*;

    pub extern "C" fn accessibilityRole(_: &Object, _: Sel) -> id {
        unsafe {
            NSString::alloc(nil).init_str("AXTextArea")
        }
    }

    pub extern "C" fn accessibilityRoleDescription(_: &Object, _: Sel) -> id {
        unsafe {
            NSString::alloc(nil).init_str("terminal")
        }
    }

    pub extern "C" fn accessibilityLabel(_: &Object, _: Sel) -> id {
        unsafe {
            NSString::alloc(nil).init_str("Terminal output")
        }
    }

    pub extern "C" fn isAccessibilityElement(_: &Object, _: Sel) -> BOOL {
        YES
    }

    pub extern "C" fn accessibilityValue(this: &Object, _: Sel) -> id {
        unsafe {
            // Get current terminal content
            let terminal_content = get_terminal_content(this);
            NSString::alloc(nil).init_str(&terminal_content)
        }
    }

    pub extern "C" fn accessibilitySelectedText(this: &Object, _: Sel) -> id {
        unsafe {
            let selected = get_selected_text(this);
            NSString::alloc(nil).init_str(&selected)
        }
    }
}
```

---

### A4. Implementation Plan

#### Phase 1: Foundation (Weeks 1-4)

1. **Smart Defaults System**
   - Create `config/src/defaults/` module
   - Implement macOS system detection
   - Define sensible default configuration
   - Test with fresh installations

2. **Swift/Rust Bridge Setup**
   - Create `macos-native/` Swift package
   - Implement FFI bridge using cbindgen
   - Set up bidirectional communication
   - Build system integration

3. **Project Structure**
   ```
   wezterm/
   ├── macos-native/
   │   ├── Package.swift
   │   ├── Sources/
   │   │   ├── WezTermBridge/      # Rust FFI
   │   │   ├── Preferences/        # SwiftUI preferences
   │   │   ├── Services/           # System services
   │   │   └── Accessibility/      # VoiceOver support
   │   └── Tests/
   ```

#### Phase 2: Metal Rendering (Weeks 5-8)

1. **Metal Backend Implementation**
   - Create `window/src/os/macos/metal.rs`
   - Implement shader pipeline
   - Create texture atlas management
   - Performance optimization

2. **Glyph Cache Adaptation**
   - Modify `wezterm-gui/src/glyphcache.rs`
   - Support Metal textures
   - Optimize cache strategies
   - ProMotion support (120Hz)

3. **Rendering Pipeline**
   - Vertex buffer management
   - Command encoder setup
   - Frame scheduling
   - Vsync handling

#### Phase 3: Native UI (Weeks 9-12)

1. **SwiftUI Preferences Panel**
   - General settings
   - Appearance customization
   - Font selection
   - Keyboard shortcuts
   - Profile management

2. **Native Dialogs**
   - File open/save dialogs
   - Color picker
   - Font picker
   - Alert dialogs

3. **Menu System Enhancement**
   - Standard macOS menus
   - Dynamic menu items
   - Keyboard shortcut management
   - Touch Bar support

#### Phase 4: System Integration (Weeks 13-16)

1. **Notification Center**
   - Bell notifications
   - Process completion
   - Background alerts

2. **Spotlight & Services**
   - Profile indexing
   - Quick Actions
   - Services menu integration

3. **iCloud Sync**
   - Settings synchronization
   - Profile sharing
   - SSH key management

#### Phase 5: Accessibility (Weeks 17-20)

1. **VoiceOver Support**
   - Screen reader compatibility
   - Cursor tracking
   - Output announcements

2. **Accessibility Inspector Compliance**
   - Role descriptions
   - Element hierarchy
   - Action support

3. **System Preferences Integration**
   - Reduced motion
   - High contrast
   - Increased contrast

---

### A5. Testing Strategy

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_macos_config() {
        let config = default_macos_config();
        assert_eq!(config.font_size, 13.0);
        assert!(config.native_macos_fullscreen_mode);
        assert!(!config.keys.is_empty());
    }

    #[test]
    fn test_system_detection() {
        let defaults = MacOSDefaults::detect();
        // Should not panic, values should be reasonable
        assert!(matches!(defaults.appearance,
            SystemAppearance::Light | SystemAppearance::Dark));
    }
}
```

#### Integration Tests
```swift
import XCTest
@testable import WezTermBridge

class SystemIntegrationTests: XCTestCase {
    func testNotificationPermission() async {
        let granted = await SystemServices.shared.requestNotificationPermission()
        // Test in appropriate environment
    }

    func testConfigBridging() {
        let rustConfig = WezTermBridge.shared.getCurrentConfig()
        XCTAssertNotNil(rustConfig)
    }
}
```

#### Performance Benchmarks
- Metal rendering FPS
- Frame time consistency
- Memory usage
- Startup time

---

## Initiative B: LibVT Library Extraction

### B1. Goals & Non-Goals

#### Goals

1. **Standalone Terminal Emulation Library**
   - Zero GUI dependencies
   - Pure terminal state management
   - Standard Rust crate interface
   - Embeddable in any application

2. **VSCode Integration Ready**
   - Node.js bindings via NAPI
   - Electron compatibility
   - WebAssembly support
   - TypeScript definitions

3. **Comprehensive Terminal Support**
   - VT100/VT220/xterm compatibility
   - 256-color and true color
   - Mouse protocol support
   - Sixel/iTerm2 images
   - OSC 8 hyperlinks

4. **High Performance**
   - Minimal allocations
   - Efficient parsing
   - Incremental rendering
   - Memory efficient

5. **Well-Documented API**
   - Comprehensive documentation
   - Examples for common use cases
   - Migration guide
   - Version compatibility

#### Non-Goals

- GUI/rendering implementation
- Platform-specific code
- Window management
- Font loading/shaping
- Input method handling

---

### B2. Architecture Overview

```
┌────────────────────────────────────────────────────────────┐
│                    LibVT Architecture                      │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ┌─────────────────────────────────────────────────────┐  │
│  │                    Public API Layer                  │  │
│  │  - Terminal creation & configuration                 │  │
│  │  - Input handling (keyboard, mouse)                  │  │
│  │  - Screen queries (cells, attributes)                │  │
│  │  - Event subscription (changes, bells, etc.)         │  │
│  └─────────────────────────────────────────────────────┘  │
│                            │                               │
│  ┌─────────────────────────────────────────────────────┐  │
│  │                  Terminal State Core                 │  │
│  │  - Screen buffer management                          │  │
│  │  - Cursor position & attributes                      │  │
│  │  - Scrollback history                                │  │
│  │  - Terminal modes (DECAWM, etc.)                     │  │
│  └─────────────────────────────────────────────────────┘  │
│                            │                               │
│  ┌─────────────────────────────────────────────────────┐  │
│  │                   Parser Engine                      │  │
│  │  - Escape sequence recognition                       │  │
│  │  - CSI/OSC/DCS/APC parsing                          │  │
│  │  - Action generation                                 │  │
│  │  - UTF-8 handling                                    │  │
│  └─────────────────────────────────────────────────────┘  │
│                            │                               │
│  ┌─────────────────────────────────────────────────────┐  │
│  │                   Cell/Surface Model                 │  │
│  │  - Cell representation                               │  │
│  │  - Line management                                   │  │
│  │  - Attribute tracking                                │  │
│  │  - Hyperlink support                                 │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                            │
└────────────────────────────────────────────────────────────┘

                    External Bindings
                           │
            ┌──────────────┼──────────────┐
            │              │              │
    ┌───────▼───────┐ ┌────▼─────┐ ┌─────▼─────┐
    │   Node.js     │ │  Python  │ │   WASM    │
    │   (NAPI-RS)   │ │ (PyO3)   │ │(wasm-pack)│
    └───────────────┘ └──────────┘ └───────────┘
```

---

### B3. API Design

#### B3.1 Core Rust API

**File:** `libvt/src/lib.rs`

```rust
//! LibVT - Terminal Emulation Library
//!
//! A high-performance terminal emulation library for building terminal
//! emulators, IDE integrations, and terminal multiplexers.

pub mod cell;
pub mod color;
pub mod config;
pub mod cursor;
pub mod events;
pub mod input;
pub mod parser;
pub mod screen;
pub mod terminal;

pub use cell::{Cell, CellAttributes};
pub use color::{ColorPalette, RgbColor};
pub use cursor::{Cursor, CursorShape};
pub use events::{TerminalEvent, EventSubscriber};
pub use input::{KeyCode, KeyModifiers, MouseButton, MouseEvent};
pub use screen::{Screen, Line};
pub use terminal::{Terminal, TerminalConfig};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Create a new terminal instance with default configuration
///
/// # Example
/// ```
/// use libvt::Terminal;
///
/// let mut term = libvt::create_terminal(80, 24);
/// term.write(b"Hello, World!\r\n");
/// ```
pub fn create_terminal(cols: u16, rows: u16) -> Terminal {
    Terminal::new(cols, rows, TerminalConfig::default())
}

/// Create a terminal with custom configuration
///
/// # Example
/// ```
/// use libvt::{Terminal, TerminalConfig};
///
/// let config = TerminalConfig {
///     scrollback_lines: 10000,
///     enable_images: true,
///     ..Default::default()
/// };
/// let mut term = libvt::create_terminal_with_config(80, 24, config);
/// ```
pub fn create_terminal_with_config(cols: u16, rows: u16, config: TerminalConfig) -> Terminal {
    Terminal::new(cols, rows, config)
}
```

#### B3.2 Terminal Core

**File:** `libvt/src/terminal.rs`

```rust
use crate::{
    cell::{Cell, CellAttributes},
    color::ColorPalette,
    cursor::{Cursor, CursorShape},
    events::{TerminalEvent, EventSubscriber},
    input::{KeyCode, KeyModifiers, MouseButton, MouseEvent},
    parser::Parser,
    screen::{Screen, Line},
};
use std::collections::VecDeque;

/// Terminal configuration
#[derive(Debug, Clone)]
pub struct TerminalConfig {
    /// Number of scrollback lines to retain
    pub scrollback_lines: usize,
    /// Enable image protocol support (Sixel, iTerm2, Kitty)
    pub enable_images: bool,
    /// Enable OSC 8 hyperlinks
    pub enable_hyperlinks: bool,
    /// Enable bracketed paste mode by default
    pub bracketed_paste: bool,
    /// Unicode version for width calculation
    pub unicode_version: UnicodeVersion,
    /// Default color palette
    pub color_palette: ColorPalette,
    /// Initial cursor shape
    pub cursor_shape: CursorShape,
    /// Enable cursor blinking
    pub cursor_blink: bool,
    /// Audible bell behavior
    pub audible_bell: bool,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            scrollback_lines: 10000,
            enable_images: true,
            enable_hyperlinks: true,
            bracketed_paste: true,
            unicode_version: UnicodeVersion::Fifteen,
            color_palette: ColorPalette::default(),
            cursor_shape: CursorShape::Block,
            cursor_blink: true,
            audible_bell: true,
        }
    }
}

/// Main terminal emulator
pub struct Terminal {
    /// Terminal dimensions
    cols: u16,
    rows: u16,

    /// Current screen (main + alternate)
    screen: Screen,

    /// Scrollback buffer
    scrollback: VecDeque<Line>,

    /// Parser for escape sequences
    parser: Parser,

    /// Cursor state
    cursor: Cursor,

    /// Configuration
    config: TerminalConfig,

    /// Event queue
    events: Vec<TerminalEvent>,

    /// Event subscribers
    subscribers: Vec<Box<dyn EventSubscriber>>,

    /// Dirty tracking
    dirty_lines: Vec<bool>,
}

impl Terminal {
    /// Create a new terminal
    pub fn new(cols: u16, rows: u16, config: TerminalConfig) -> Self {
        let screen = Screen::new(cols as usize, rows as usize);
        let scrollback = VecDeque::with_capacity(config.scrollback_lines);

        Self {
            cols,
            rows,
            screen,
            scrollback,
            parser: Parser::new(),
            cursor: Cursor::new(),
            config,
            events: Vec::new(),
            subscribers: Vec::new(),
            dirty_lines: vec![false; rows as usize],
        }
    }

    /// Write data to the terminal
    ///
    /// This processes the input through the escape sequence parser
    /// and updates terminal state accordingly.
    pub fn write(&mut self, data: &[u8]) -> usize {
        let actions = self.parser.parse(data);

        for action in actions {
            self.perform_action(action);
        }

        data.len()
    }

    /// Resize the terminal
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        self.screen.resize(cols as usize, rows as usize);
        self.dirty_lines = vec![true; rows as usize];
        self.emit_event(TerminalEvent::Resized { cols, rows });
    }

    /// Handle keyboard input
    pub fn key_down(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Vec<u8> {
        self.encode_key(key, modifiers)
    }

    /// Handle mouse event
    pub fn mouse_event(&mut self, event: MouseEvent) -> Vec<u8> {
        self.encode_mouse(event)
    }

    /// Get a cell at the specified position
    pub fn get_cell(&self, col: u16, row: u16) -> Option<&Cell> {
        self.screen.get_cell(col as usize, row as usize)
    }

    /// Get a line at the specified row
    pub fn get_line(&self, row: u16) -> Option<&Line> {
        self.screen.get_line(row as usize)
    }

    /// Get all visible lines
    pub fn visible_lines(&self) -> impl Iterator<Item = &Line> {
        self.screen.lines()
    }

    /// Get current cursor position
    pub fn cursor_position(&self) -> (u16, u16) {
        (self.cursor.col, self.cursor.row)
    }

    /// Get cursor state
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// Get terminal dimensions
    pub fn size(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    /// Get scrollback buffer
    pub fn scrollback(&self) -> &VecDeque<Line> {
        &self.scrollback
    }

    /// Get dirty lines (lines that changed since last check)
    pub fn dirty_lines(&self) -> &[bool] {
        &self.dirty_lines
    }

    /// Clear dirty tracking
    pub fn clear_dirty(&mut self) {
        self.dirty_lines.fill(false);
    }

    /// Subscribe to terminal events
    pub fn subscribe(&mut self, subscriber: Box<dyn EventSubscriber>) {
        self.subscribers.push(subscriber);
    }

    /// Get pending events
    pub fn take_events(&mut self) -> Vec<TerminalEvent> {
        std::mem::take(&mut self.events)
    }

    /// Set color palette
    pub fn set_palette(&mut self, palette: ColorPalette) {
        self.config.color_palette = palette;
        self.emit_event(TerminalEvent::PaletteChanged);
    }

    /// Get current color palette
    pub fn palette(&self) -> &ColorPalette {
        &self.config.color_palette
    }

    /// Get terminal title (set via OSC sequences)
    pub fn title(&self) -> &str {
        self.screen.title()
    }

    /// Get current working directory (set via OSC 7)
    pub fn working_directory(&self) -> Option<&str> {
        self.screen.working_directory()
    }

    /// Check if terminal is in alternate screen mode
    pub fn is_alternate_screen(&self) -> bool {
        self.screen.is_alternate()
    }

    /// Get selection (if any)
    pub fn selection(&self) -> Option<Selection> {
        self.screen.selection()
    }

    /// Set selection
    pub fn set_selection(&mut self, start: Position, end: Position) {
        self.screen.set_selection(start, end);
        self.emit_event(TerminalEvent::SelectionChanged);
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.screen.clear_selection();
        self.emit_event(TerminalEvent::SelectionChanged);
    }

    /// Get selected text
    pub fn selected_text(&self) -> Option<String> {
        self.screen.selected_text()
    }

    // Private methods

    fn perform_action(&mut self, action: Action) {
        match action {
            Action::Print(c) => self.print(c),
            Action::Control(ctrl) => self.control(ctrl),
            Action::Csi(csi) => self.csi(csi),
            Action::Esc(esc) => self.esc(esc),
            Action::Osc(osc) => self.osc(osc),
            Action::Dcs(dcs) => self.dcs(dcs),
            Action::Apc(apc) => self.apc(apc),
        }
    }

    fn emit_event(&mut self, event: TerminalEvent) {
        self.events.push(event.clone());

        for subscriber in &self.subscribers {
            subscriber.on_event(&event);
        }
    }

    fn encode_key(&self, key: KeyCode, modifiers: KeyModifiers) -> Vec<u8> {
        // Key encoding logic based on terminal mode
        // (application cursor keys, etc.)
        todo!()
    }

    fn encode_mouse(&self, event: MouseEvent) -> Vec<u8> {
        // Mouse encoding based on enabled protocols
        // (SGR, URXVT, X10, etc.)
        todo!()
    }
}

/// Selection in terminal
#[derive(Debug, Clone, Copy)]
pub struct Selection {
    pub start: Position,
    pub end: Position,
}

/// Position in terminal
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub col: u16,
    pub row: u16,
}
```

#### B3.3 Event System

**File:** `libvt/src/events.rs`

```rust
/// Terminal events that can be subscribed to
#[derive(Debug, Clone)]
pub enum TerminalEvent {
    /// Terminal was resized
    Resized { cols: u16, rows: u16 },

    /// Terminal bell rang
    Bell,

    /// Title changed (via OSC 0/2)
    TitleChanged(String),

    /// Working directory changed (via OSC 7)
    WorkingDirectoryChanged(String),

    /// Color palette changed
    PaletteChanged,

    /// Cursor position changed
    CursorMoved { col: u16, row: u16 },

    /// Cursor shape changed
    CursorShapeChanged(CursorShape),

    /// Cursor visibility changed
    CursorVisibilityChanged(bool),

    /// Selection changed
    SelectionChanged,

    /// Clipboard request (OSC 52)
    ClipboardRequest { clipboard: ClipboardType, data: String },

    /// Hyperlink activated
    HyperlinkActivated { url: String },

    /// Image loaded (Sixel/iTerm2/Kitty)
    ImageLoaded { id: u32 },

    /// Output written to screen
    OutputWritten { lines_affected: Vec<u16> },

    /// Mode changed
    ModeChanged { mode: TerminalMode, enabled: bool },

    /// Color query response (OSC 10/11)
    ColorResponse { index: u8, color: RgbColor },
}

/// Event subscriber trait
pub trait EventSubscriber: Send + Sync {
    fn on_event(&self, event: &TerminalEvent);
}

/// Clipboard type
#[derive(Debug, Clone, Copy)]
pub enum ClipboardType {
    Clipboard,
    Primary,
    Secondary,
}

/// Terminal modes
#[derive(Debug, Clone, Copy)]
pub enum TerminalMode {
    CursorKeys,      // DECCKM
    Insert,          // IRM
    SendReceive,     // SRM
    AutoWrap,        // DECAWM
    CursorBlink,     // ATT610
    BracketedPaste,  // DEC private mode
    MouseTracking,   // Mouse modes
    FocusTracking,   // Focus in/out events
    AlternateScreen, // Alternate screen buffer
}
```

#### B3.4 Cell and Color Model

**File:** `libvt/src/cell.rs`

```rust
use crate::color::{ColorSpec, RgbColor};

/// A single terminal cell
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    /// Characters in this cell (may be multiple for wide chars)
    text: String,
    /// Cell attributes
    attrs: CellAttributes,
    /// Hyperlink (if any)
    hyperlink: Option<Hyperlink>,
}

impl Cell {
    /// Create a new cell with the given character
    pub fn new(ch: char) -> Self {
        Self {
            text: ch.to_string(),
            attrs: CellAttributes::default(),
            hyperlink: None,
        }
    }

    /// Create an empty cell
    pub fn blank() -> Self {
        Self {
            text: " ".to_string(),
            attrs: CellAttributes::default(),
            hyperlink: None,
        }
    }

    /// Get the text content
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get cell attributes
    pub fn attrs(&self) -> &CellAttributes {
        &self.attrs
    }

    /// Get hyperlink if present
    pub fn hyperlink(&self) -> Option<&Hyperlink> {
        self.hyperlink.as_ref()
    }

    /// Check if cell is blank
    pub fn is_blank(&self) -> bool {
        self.text.trim().is_empty() && self.hyperlink.is_none()
    }

    /// Get the width of this cell (1 for normal, 2 for wide)
    pub fn width(&self) -> u8 {
        // Unicode width calculation
        unicode_width::UnicodeWidthStr::width(self.text.as_str()) as u8
    }
}

/// Cell attributes (colors, styles)
#[derive(Debug, Clone, PartialEq)]
pub struct CellAttributes {
    /// Foreground color
    pub foreground: ColorSpec,
    /// Background color
    pub background: ColorSpec,
    /// Bold/Bright
    pub bold: bool,
    /// Italic
    pub italic: bool,
    /// Underline style
    pub underline: UnderlineStyle,
    /// Underline color (if different from foreground)
    pub underline_color: Option<ColorSpec>,
    /// Strikethrough
    pub strikethrough: bool,
    /// Blink
    pub blink: bool,
    /// Reverse video
    pub reverse: bool,
    /// Hidden/Invisible
    pub hidden: bool,
    /// Dim/Faint
    pub dim: bool,
}

impl Default for CellAttributes {
    fn default() -> Self {
        Self {
            foreground: ColorSpec::Default,
            background: ColorSpec::Default,
            bold: false,
            italic: false,
            underline: UnderlineStyle::None,
            underline_color: None,
            strikethrough: false,
            blink: false,
            reverse: false,
            hidden: false,
            dim: false,
        }
    }
}

/// Underline styles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnderlineStyle {
    None,
    Single,
    Double,
    Curly,
    Dotted,
    Dashed,
}

/// Hyperlink information (OSC 8)
#[derive(Debug, Clone, PartialEq)]
pub struct Hyperlink {
    /// URL of the hyperlink
    pub url: String,
    /// Optional ID for tracking
    pub id: Option<String>,
    /// Additional parameters
    pub params: Vec<(String, String)>,
}
```

**File:** `libvt/src/color.rs`

```rust
/// Color specification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSpec {
    /// Default terminal color
    Default,
    /// ANSI color (0-15)
    Ansi(u8),
    /// 256-color palette index
    Palette(u8),
    /// True color RGB
    Rgb(RgbColor),
}

/// RGB color
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Self { r, g, b })
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// Color palette (16 ANSI colors)
#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub colors: [RgbColor; 256],
}

impl Default for ColorPalette {
    fn default() -> Self {
        let mut colors = [RgbColor::new(0, 0, 0); 256];

        // Standard ANSI colors
        colors[0] = RgbColor::new(0, 0, 0);       // Black
        colors[1] = RgbColor::new(204, 0, 0);     // Red
        colors[2] = RgbColor::new(78, 154, 6);    // Green
        colors[3] = RgbColor::new(196, 160, 0);   // Yellow
        colors[4] = RgbColor::new(52, 101, 164);  // Blue
        colors[5] = RgbColor::new(117, 80, 123);  // Magenta
        colors[6] = RgbColor::new(6, 152, 154);   // Cyan
        colors[7] = RgbColor::new(211, 215, 207); // White

        // Bright colors
        colors[8] = RgbColor::new(85, 87, 83);    // Bright Black
        colors[9] = RgbColor::new(239, 41, 41);   // Bright Red
        colors[10] = RgbColor::new(138, 226, 52); // Bright Green
        colors[11] = RgbColor::new(252, 233, 79); // Bright Yellow
        colors[12] = RgbColor::new(114, 159, 207);// Bright Blue
        colors[13] = RgbColor::new(173, 127, 168);// Bright Magenta
        colors[14] = RgbColor::new(52, 226, 226); // Bright Cyan
        colors[15] = RgbColor::new(238, 238, 236);// Bright White

        // 216 color cube (16-231)
        let mut idx = 16;
        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    let rv = if r == 0 { 0 } else { 55 + r * 40 };
                    let gv = if g == 0 { 0 } else { 55 + g * 40 };
                    let bv = if b == 0 { 0 } else { 55 + b * 40 };
                    colors[idx] = RgbColor::new(rv, gv, bv);
                    idx += 1;
                }
            }
        }

        // Grayscale (232-255)
        for i in 0..24 {
            let v = 8 + i * 10;
            colors[232 + i as usize] = RgbColor::new(v, v, v);
        }

        Self { colors }
    }
}
```

#### B3.5 Node.js Bindings (NAPI-RS)

**File:** `libvt-node/src/lib.rs`

```rust
use napi::bindgen_prelude::*;
use napi_derive::napi;
use libvt::{Terminal, TerminalConfig};

#[napi(object)]
pub struct JsTerminalConfig {
    pub scrollback_lines: Option<u32>,
    pub enable_images: Option<bool>,
    pub enable_hyperlinks: Option<bool>,
}

#[napi]
pub struct JsTerminal {
    inner: Terminal,
}

#[napi]
impl JsTerminal {
    #[napi(constructor)]
    pub fn new(cols: u16, rows: u16, config: Option<JsTerminalConfig>) -> Self {
        let config = config.map(|c| TerminalConfig {
            scrollback_lines: c.scrollback_lines.unwrap_or(10000) as usize,
            enable_images: c.enable_images.unwrap_or(true),
            enable_hyperlinks: c.enable_hyperlinks.unwrap_or(true),
            ..Default::default()
        }).unwrap_or_default();

        Self {
            inner: Terminal::new(cols, rows, config),
        }
    }

    #[napi]
    pub fn write(&mut self, data: Buffer) -> u32 {
        self.inner.write(&data) as u32
    }

    #[napi]
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.inner.resize(cols, rows);
    }

    #[napi]
    pub fn get_cell(&self, col: u16, row: u16) -> Option<JsCell> {
        self.inner.get_cell(col, row).map(|cell| JsCell {
            text: cell.text().to_string(),
            fg_color: cell.attrs().foreground.to_css(),
            bg_color: cell.attrs().background.to_css(),
            bold: cell.attrs().bold,
            italic: cell.attrs().italic,
            underline: cell.attrs().underline != libvt::cell::UnderlineStyle::None,
        })
    }

    #[napi]
    pub fn cursor_position(&self) -> JsCursorPosition {
        let (col, row) = self.inner.cursor_position();
        JsCursorPosition { col, row }
    }

    #[napi]
    pub fn title(&self) -> String {
        self.inner.title().to_string()
    }

    #[napi]
    pub fn take_events(&mut self) -> Vec<JsTerminalEvent> {
        self.inner.take_events().into_iter().map(|e| e.into()).collect()
    }
}

#[napi(object)]
pub struct JsCell {
    pub text: String,
    pub fg_color: String,
    pub bg_color: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[napi(object)]
pub struct JsCursorPosition {
    pub col: u16,
    pub row: u16,
}

#[napi]
pub enum JsTerminalEvent {
    Bell,
    TitleChanged,
    Resized,
    CursorMoved,
}
```

#### B3.6 WebAssembly Bindings

**File:** `libvt-wasm/src/lib.rs`

```rust
use wasm_bindgen::prelude::*;
use libvt::{Terminal, TerminalConfig};

#[wasm_bindgen]
pub struct WasmTerminal {
    inner: Terminal,
}

#[wasm_bindgen]
impl WasmTerminal {
    #[wasm_bindgen(constructor)]
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            inner: Terminal::new(cols, rows, TerminalConfig::default()),
        }
    }

    #[wasm_bindgen]
    pub fn write(&mut self, data: &[u8]) -> usize {
        self.inner.write(data)
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.inner.resize(cols, rows);
    }

    #[wasm_bindgen(js_name = getCellText)]
    pub fn get_cell_text(&self, col: u16, row: u16) -> Option<String> {
        self.inner.get_cell(col, row).map(|c| c.text().to_string())
    }

    #[wasm_bindgen(js_name = getLineText)]
    pub fn get_line_text(&self, row: u16) -> Option<String> {
        self.inner.get_line(row).map(|line| {
            line.cells().map(|c| c.text()).collect()
        })
    }

    #[wasm_bindgen(js_name = cursorCol)]
    pub fn cursor_col(&self) -> u16 {
        self.inner.cursor_position().0
    }

    #[wasm_bindgen(js_name = cursorRow)]
    pub fn cursor_row(&self) -> u16 {
        self.inner.cursor_position().1
    }
}
```

---

### B4. Implementation Plan

#### Phase 1: Core Extraction (Weeks 1-4)

1. **Create libvt crate structure**
   ```
   libvt/
   ├── Cargo.toml
   ├── src/
   │   ├── lib.rs
   │   ├── terminal.rs
   │   ├── cell.rs
   │   ├── color.rs
   │   ├── cursor.rs
   │   ├── events.rs
   │   ├── input.rs
   │   ├── parser/
   │   │   ├── mod.rs
   │   │   ├── csi.rs
   │   │   ├── osc.rs
   │   │   ├── esc.rs
   │   │   └── dcs.rs
   │   └── screen/
   │       ├── mod.rs
   │       └── line.rs
   ```

2. **Extract from existing WezTerm code**
   - `term/` → `libvt/src/terminal.rs`
   - `wezterm-escape-parser/` → `libvt/src/parser/`
   - `wezterm-cell/` → `libvt/src/cell.rs`
   - `wezterm-surface/` → `libvt/src/screen/`

3. **Remove GUI dependencies**
   - No window system references
   - No font/rendering code
   - Pure terminal state management

#### Phase 2: API Stabilization (Weeks 5-8)

1. **Design stable public API**
   - Terminal creation and configuration
   - Input/output interfaces
   - Event subscription system
   - Query methods for state

2. **Documentation**
   - Comprehensive API docs
   - Usage examples
   - Migration guide from WezTerm internals

3. **Testing**
   - Unit tests for all public APIs
   - Property-based testing
   - Compatibility tests (vttest, etc.)

#### Phase 3: Language Bindings (Weeks 9-12)

1. **Node.js (NAPI-RS)**
   - Complete bindings
   - TypeScript definitions
   - NPM package setup
   - VSCode extension example

2. **WebAssembly (wasm-bindgen)**
   - WASM build setup
   - JavaScript glue code
   - Browser demo

3. **Python (PyO3)**
   - Python bindings
   - PyPI package
   - Jupyter notebook example

#### Phase 4: WezTerm Migration (Weeks 13-16)

1. **Integrate libvt back into WezTerm**
   - Replace `term/` with libvt dependency
   - Update rendering pipeline
   - Maintain compatibility

2. **Performance optimization**
   - Benchmark against original
   - Memory optimization
   - Parsing performance

3. **Release preparation**
   - Version 1.0 release
   - crates.io publication
   - npm package publication

---

### B5. Testing Strategy

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_creation() {
        let term = create_terminal(80, 24);
        assert_eq!(term.size(), (80, 24));
        assert_eq!(term.cursor_position(), (0, 0));
    }

    #[test]
    fn test_simple_write() {
        let mut term = create_terminal(80, 24);
        term.write(b"Hello");

        assert_eq!(term.get_cell(0, 0).unwrap().text(), "H");
        assert_eq!(term.get_cell(1, 0).unwrap().text(), "e");
        assert_eq!(term.cursor_position(), (5, 0));
    }

    #[test]
    fn test_newline() {
        let mut term = create_terminal(80, 24);
        term.write(b"Hello\r\nWorld");

        assert_eq!(term.get_line(0).unwrap().to_string().trim(), "Hello");
        assert_eq!(term.get_line(1).unwrap().to_string().trim(), "World");
    }

    #[test]
    fn test_colors() {
        let mut term = create_terminal(80, 24);
        // Set red foreground: ESC[31m
        term.write(b"\x1b[31mRed");

        let cell = term.get_cell(0, 0).unwrap();
        assert_eq!(cell.attrs().foreground, ColorSpec::Ansi(1));
    }

    #[test]
    fn test_cursor_movement() {
        let mut term = create_terminal(80, 24);
        // Move cursor to row 5, col 10: ESC[5;10H
        term.write(b"\x1b[5;10H");

        assert_eq!(term.cursor_position(), (9, 4)); // 0-indexed
    }
}
```

#### Integration Tests

```rust
#[test]
fn test_vttest_compatibility() {
    let mut term = create_terminal(80, 24);

    // Test various VT escape sequences
    // ... vttest cases
}

#[test]
fn test_image_protocol() {
    let mut term = create_terminal_with_config(80, 24, TerminalConfig {
        enable_images: true,
        ..Default::default()
    });

    // Test Sixel, iTerm2, Kitty image protocols
}
```

#### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_write_doesnt_panic(data in any::<Vec<u8>>()) {
        let mut term = create_terminal(80, 24);
        term.write(&data); // Should never panic
    }

    #[test]
    fn test_resize_maintains_invariants(cols in 1u16..1000, rows in 1u16..1000) {
        let mut term = create_terminal(80, 24);
        term.resize(cols, rows);
        assert_eq!(term.size(), (cols, rows));
    }
}
```

---

## Shared Infrastructure

### Rust/Swift FFI Bridge

**File:** `macos-native/Sources/WezTermBridge/bridge.h`

```c
#ifndef WEZTERM_BRIDGE_H
#define WEZTERM_BRIDGE_H

#include <stdint.h>
#include <stdbool.h>

// Opaque pointers
typedef struct WezTermConfig WezTermConfig;
typedef struct WezTermTerminal WezTermTerminal;

// Configuration API
WezTermConfig* wezterm_config_new(void);
void wezterm_config_free(WezTermConfig* config);
void wezterm_config_set_font_size(WezTermConfig* config, double size);
void wezterm_config_set_color_scheme(WezTermConfig* config, const char* name);
bool wezterm_config_apply(WezTermConfig* config);

// System detection
const char* wezterm_detect_appearance(void);
const char* wezterm_detect_accent_color(void);
bool wezterm_detect_reduced_motion(void);

// Terminal API
WezTermTerminal* wezterm_terminal_new(uint16_t cols, uint16_t rows);
void wezterm_terminal_free(WezTermTerminal* term);
size_t wezterm_terminal_write(WezTermTerminal* term, const uint8_t* data, size_t len);
void wezterm_terminal_resize(WezTermTerminal* term, uint16_t cols, uint16_t rows);

#endif
```

**File:** `src/macos_bridge.rs`

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn wezterm_config_new() -> *mut Config {
    Box::into_raw(Box::new(Config::default()))
}

#[no_mangle]
pub unsafe extern "C" fn wezterm_config_free(config: *mut Config) {
    if !config.is_null() {
        drop(Box::from_raw(config));
    }
}

#[no_mangle]
pub unsafe extern "C" fn wezterm_config_set_font_size(config: *mut Config, size: f64) {
    if !config.is_null() {
        (*config).font_size = size;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wezterm_detect_appearance() -> *mut c_char {
    let appearance = if is_dark_mode() { "dark" } else { "light" };
    CString::new(appearance).unwrap().into_raw()
}
```

---

## Migration Strategy

### For Existing WezTerm Users

1. **Configuration Compatibility**
   - All existing Lua configurations remain valid
   - New GUI preferences are optional
   - Gradual migration path

2. **Rendering Backend**
   - OpenGL remains default
   - Metal opt-in initially
   - Automatic selection based on performance

3. **Feature Flags**
   ```toml
   [features]
   metal-rendering = []
   native-preferences = []
   smart-defaults = ["native-preferences"]
   ```

### For LibVT Adopters

1. **Drop-in Replacement**
   - Similar API to xterm.js, vte.rs
   - Comprehensive documentation
   - Example projects

2. **VSCode Integration**
   ```typescript
   import { Terminal } from '@libvt/node';

   const term = new Terminal(80, 24);
   term.write(Buffer.from('Hello, World!\r\n'));
   ```

---

## Risk Assessment

### Technical Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| Metal rendering performance regression | High | Medium | Extensive benchmarking, fallback to OpenGL |
| Swift/Rust FFI complexity | Medium | High | Incremental development, thorough testing |
| LibVT API breaking changes | High | Medium | Semantic versioning, deprecation policy |
| Memory safety in FFI | High | Low | Careful ownership management, fuzzing |

### Schedule Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| Scope creep | High | High | Strict phase gates, MVP focus |
| Platform-specific bugs | Medium | High | CI matrix testing, beta program |
| Documentation delays | Medium | Medium | Doc-first development |

---

## Success Metrics

### macOS Native Experience

- **Zero-config score**: 90% of new users don't create wezterm.lua
- **Performance**: Metal rendering matches or beats OpenGL
- **Accessibility**: 100% VoiceOver compatibility
- **System integration**: All major macOS services integrated
- **User satisfaction**: 4.5+ stars on App Store

### LibVT Library

- **Adoption**: 1000+ npm downloads/month within 6 months
- **VSCode integration**: Official VSCode terminal option
- **API stability**: No breaking changes after 1.0
- **Performance**: <1ms parse time for typical output
- **Compatibility**: 99% vttest pass rate

---

## Timeline Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Foundation | Weeks 1-4 | Smart defaults, Swift bridge |
| Metal Rendering | Weeks 5-8 | Metal backend, ProMotion |
| Native UI | Weeks 9-12 | SwiftUI preferences, native dialogs |
| System Integration | Weeks 13-16 | Services, Spotlight, iCloud |
| Accessibility | Weeks 17-20 | VoiceOver, Inspector compliance |
| LibVT Core | Weeks 1-4 | Extracted library |
| API Stabilization | Weeks 5-8 | 1.0 API, documentation |
| Language Bindings | Weeks 9-12 | Node.js, WASM, Python |
| WezTerm Migration | Weeks 13-16 | Integrated back |

**Total Timeline:** 20 weeks (5 months) for full implementation of both initiatives.

---

## Conclusion

This specification provides a comprehensive roadmap for transforming WezTerm into a first-class macOS application while simultaneously extracting its powerful terminal emulation core into a standalone library. The dual approach ensures WezTerm remains competitive in the terminal emulator space while enabling broader adoption of its technology across the development ecosystem.

The modular architecture, careful API design, and phased implementation plan minimize risk while delivering significant value at each milestone. Success depends on maintaining backward compatibility, thorough testing, and continuous user feedback throughout development.
