/*
 * WezTerm Native macOS Preferences UI
 *
 * SwiftUI-based preferences panel for configuring WezTerm.
 */

import SwiftUI

// MARK: - Main Preferences View

@available(macOS 11.0, *)
public struct PreferencesView: View {
    @StateObject private var viewModel = PreferencesViewModel()

    public init() {}

    public var body: some View {
        TabView {
            GeneralPreferencesView(viewModel: viewModel)
                .tabItem {
                    Label("General", systemImage: "gear")
                }
                .tag(0)

            AppearancePreferencesView(viewModel: viewModel)
                .tabItem {
                    Label("Appearance", systemImage: "paintbrush")
                }
                .tag(1)

            FontPreferencesView(viewModel: viewModel)
                .tabItem {
                    Label("Fonts", systemImage: "textformat")
                }
                .tag(2)

            WindowPreferencesView(viewModel: viewModel)
                .tabItem {
                    Label("Window", systemImage: "macwindow")
                }
                .tag(3)

            AdvancedPreferencesView(viewModel: viewModel)
                .tabItem {
                    Label("Advanced", systemImage: "gearshape.2")
                }
                .tag(4)
        }
        .frame(width: 600, height: 450)
        .padding()
    }
}

// MARK: - General Preferences

@available(macOS 11.0, *)
struct GeneralPreferencesView: View {
    @ObservedObject var viewModel: PreferencesViewModel

    var body: some View {
        Form {
            Section(header: Text("Startup")) {
                Toggle("Open new window on launch", isOn: $viewModel.openWindowOnLaunch)
                Toggle("Restore windows when re-opening app", isOn: $viewModel.restoreWindows)
            }

            Section(header: Text("Tabs")) {
                Toggle("Show tab bar", isOn: $viewModel.showTabBar)
                Toggle("Hide tab bar when only one tab", isOn: $viewModel.hideTabBarIfOnlyOneTab)
                    .disabled(!viewModel.showTabBar)

                Picker("Tab bar position:", selection: $viewModel.tabBarPosition) {
                    Text("Top").tag(TabBarPosition.top)
                    Text("Bottom").tag(TabBarPosition.bottom)
                }
                .disabled(!viewModel.showTabBar)

                Toggle("Use fancy tab bar", isOn: $viewModel.useFancyTabBar)
                    .disabled(!viewModel.showTabBar)
            }

            Section(header: Text("Shell")) {
                TextField("Default shell:", text: $viewModel.defaultShell)
                TextField("Initial working directory:", text: $viewModel.initialDirectory)
            }

            Section(header: Text("Behavior")) {
                Toggle("Confirm before closing window with processes", isOn: $viewModel.confirmOnQuit)
                Toggle("Use native macOS full screen", isOn: $viewModel.nativeFullScreen)
            }

            Section(header: Text("Updates")) {
                Toggle("Check for updates automatically", isOn: $viewModel.checkForUpdates)
            }
        }
        .padding()
    }
}

// MARK: - Appearance Preferences

@available(macOS 11.0, *)
struct AppearancePreferencesView: View {
    @ObservedObject var viewModel: PreferencesViewModel

    var body: some View {
        Form {
            Section(header: Text("Theme")) {
                Picker("Color Scheme:", selection: $viewModel.colorScheme) {
                    ForEach(viewModel.availableColorSchemes, id: \.self) { scheme in
                        Text(scheme)
                    }
                }

                Toggle("Automatically adjust for system appearance", isOn: $viewModel.adjustForSystemAppearance)
            }

            Section(header: Text("Transparency")) {
                VStack(alignment: .leading) {
                    Text("Window Opacity: \(Int(viewModel.windowOpacity * 100))%")
                    Slider(value: $viewModel.windowOpacity, in: 0.5...1.0, step: 0.05)
                }

                VStack(alignment: .leading) {
                    Text("Background Blur: \(Int(viewModel.backgroundBlur))")
                    Slider(value: $viewModel.backgroundBlur, in: 0...30, step: 1)
                }
            }

            Section(header: Text("Cursor")) {
                Picker("Cursor style:", selection: $viewModel.cursorStyle) {
                    Text("Block").tag(CursorStyleChoice.block)
                    Text("Underline").tag(CursorStyleChoice.underline)
                    Text("Bar").tag(CursorStyleChoice.bar)
                }

                Toggle("Cursor blinks", isOn: $viewModel.cursorBlinks)
            }

            Section(header: Text("Scrollback")) {
                HStack {
                    Text("Scrollback lines:")
                    TextField("", value: $viewModel.scrollbackLines, formatter: NumberFormatter())
                        .frame(width: 100)
                }
            }
        }
        .padding()
    }
}

// MARK: - Font Preferences

@available(macOS 11.0, *)
struct FontPreferencesView: View {
    @ObservedObject var viewModel: PreferencesViewModel

    var body: some View {
        Form {
            Section(header: Text("Font")) {
                HStack {
                    Text("Font Family:")
                    Spacer()
                    Menu(viewModel.fontFamily) {
                        Button("SF Mono") { viewModel.fontFamily = "SF Mono" }
                        Button("Menlo") { viewModel.fontFamily = "Menlo" }
                        Button("Monaco") { viewModel.fontFamily = "Monaco" }
                        Button("JetBrains Mono") { viewModel.fontFamily = "JetBrains Mono" }
                        Button("Fira Code") { viewModel.fontFamily = "Fira Code" }
                        Button("Source Code Pro") { viewModel.fontFamily = "Source Code Pro" }
                        Divider()
                        Button("Other...") {
                            viewModel.showFontPicker = true
                        }
                    }
                }

                HStack {
                    Text("Font Size:")
                    Slider(value: $viewModel.fontSize, in: 8...36, step: 0.5)
                    Text("\(viewModel.fontSize, specifier: "%.1f") pt")
                        .frame(width: 60)
                }

                HStack {
                    Text("Font Weight:")
                    Picker("", selection: $viewModel.fontWeight) {
                        Text("Thin").tag(100)
                        Text("Extra Light").tag(200)
                        Text("Light").tag(300)
                        Text("Regular").tag(400)
                        Text("Medium").tag(500)
                        Text("Semi Bold").tag(600)
                        Text("Bold").tag(700)
                        Text("Extra Bold").tag(800)
                        Text("Black").tag(900)
                    }
                }
            }

            Section(header: Text("Spacing")) {
                HStack {
                    Text("Line Height:")
                    Slider(value: $viewModel.lineHeight, in: 0.8...2.0, step: 0.05)
                    Text("\(viewModel.lineHeight, specifier: "%.2f")")
                        .frame(width: 50)
                }

                HStack {
                    Text("Cell Width:")
                    Slider(value: $viewModel.cellWidth, in: 0.8...1.5, step: 0.05)
                    Text("\(viewModel.cellWidth, specifier: "%.2f")")
                        .frame(width: 50)
                }
            }

            Section(header: Text("Rendering")) {
                Toggle("Use ligatures", isOn: $viewModel.useLigatures)
                Toggle("Use bold font for bold text", isOn: $viewModel.useBoldFont)

                Picker("Font smoothing:", selection: $viewModel.fontSmoothing) {
                    Text("System Default").tag(FontSmoothingChoice.system)
                    Text("None").tag(FontSmoothingChoice.none)
                    Text("Light").tag(FontSmoothingChoice.light)
                    Text("Medium").tag(FontSmoothingChoice.medium)
                    Text("Strong").tag(FontSmoothingChoice.strong)
                }
            }

            Section {
                HStack {
                    Spacer()
                    Text("Sample: The quick brown fox jumps over the lazy dog")
                        .font(.system(size: CGFloat(viewModel.fontSize), weight: fontWeight(viewModel.fontWeight), design: .monospaced))
                    Spacer()
                }
            }
        }
        .padding()
    }

    private func fontWeight(_ weight: Int) -> Font.Weight {
        switch weight {
        case 100: return .ultraLight
        case 200: return .thin
        case 300: return .light
        case 400: return .regular
        case 500: return .medium
        case 600: return .semibold
        case 700: return .bold
        case 800: return .heavy
        case 900: return .black
        default: return .regular
        }
    }
}

// MARK: - Window Preferences

@available(macOS 11.0, *)
struct WindowPreferencesView: View {
    @ObservedObject var viewModel: PreferencesViewModel

    var body: some View {
        Form {
            Section(header: Text("Window Size")) {
                HStack {
                    Text("Initial columns:")
                    TextField("", value: $viewModel.initialCols, formatter: NumberFormatter())
                        .frame(width: 80)
                }

                HStack {
                    Text("Initial rows:")
                    TextField("", value: $viewModel.initialRows, formatter: NumberFormatter())
                        .frame(width: 80)
                }
            }

            Section(header: Text("Window Behavior")) {
                Toggle("Remember window position", isOn: $viewModel.rememberWindowPosition)
                Toggle("Remember window size", isOn: $viewModel.rememberWindowSize)
            }

            Section(header: Text("Title")) {
                Toggle("Show terminal title in tab", isOn: $viewModel.showTitleInTab)
                Toggle("Show working directory in title", isOn: $viewModel.showDirectoryInTitle)
            }

            Section(header: Text("Padding")) {
                HStack {
                    Text("Horizontal padding:")
                    TextField("", value: $viewModel.horizontalPadding, formatter: NumberFormatter())
                        .frame(width: 80)
                    Text("px")
                }

                HStack {
                    Text("Vertical padding:")
                    TextField("", value: $viewModel.verticalPadding, formatter: NumberFormatter())
                        .frame(width: 80)
                    Text("px")
                }
            }
        }
        .padding()
    }
}

// MARK: - Advanced Preferences

@available(macOS 11.0, *)
struct AdvancedPreferencesView: View {
    @ObservedObject var viewModel: PreferencesViewModel

    var body: some View {
        Form {
            Section(header: Text("Terminal")) {
                TextField("TERM environment variable:", text: $viewModel.termEnvVar)

                Toggle("Enable Kitty keyboard protocol", isOn: $viewModel.enableKittyKeyboard)
                Toggle("Enable Kitty graphics protocol", isOn: $viewModel.enableKittyGraphics)
            }

            Section(header: Text("Performance")) {
                Toggle("Use Metal rendering (experimental)", isOn: $viewModel.useMetalRendering)

                HStack {
                    Text("Max FPS:")
                    TextField("", value: $viewModel.maxFps, formatter: NumberFormatter())
                        .frame(width: 80)
                }
            }

            Section(header: Text("Configuration")) {
                HStack {
                    Text("Config file:")
                    Text(viewModel.configFilePath)
                        .foregroundColor(.secondary)
                    Spacer()
                    Button("Open") {
                        viewModel.openConfigFile()
                    }
                }

                Button("Reload Configuration") {
                    viewModel.reloadConfiguration()
                }

                Button("Reset to Defaults") {
                    viewModel.resetToDefaults()
                }
                .foregroundColor(.red)
            }

            Section(header: Text("Debug")) {
                Toggle("Log unknown escape sequences", isOn: $viewModel.logUnknownEscapes)
                Toggle("Enable debug logging", isOn: $viewModel.enableDebugLogging)
            }
        }
        .padding()
    }
}

// MARK: - View Model

@available(macOS 11.0, *)
@MainActor
class PreferencesViewModel: ObservableObject {
    // General
    @Published var openWindowOnLaunch: Bool = true
    @Published var restoreWindows: Bool = true
    @Published var showTabBar: Bool = true
    @Published var hideTabBarIfOnlyOneTab: Bool = false
    @Published var tabBarPosition: TabBarPosition = .top
    @Published var useFancyTabBar: Bool = true
    @Published var defaultShell: String = ""
    @Published var initialDirectory: String = ""
    @Published var confirmOnQuit: Bool = true
    @Published var nativeFullScreen: Bool = true
    @Published var checkForUpdates: Bool = true

    // Appearance
    @Published var colorScheme: String = "Builtin Solarized Dark"
    @Published var adjustForSystemAppearance: Bool = true
    @Published var windowOpacity: Double = 0.95
    @Published var backgroundBlur: Double = 10
    @Published var cursorStyle: CursorStyleChoice = .block
    @Published var cursorBlinks: Bool = true
    @Published var scrollbackLines: Int = 10000

    // Font
    @Published var fontFamily: String = "SF Mono"
    @Published var fontSize: Double = 13.0
    @Published var fontWeight: Int = 400
    @Published var lineHeight: Double = 1.1
    @Published var cellWidth: Double = 1.0
    @Published var useLigatures: Bool = false
    @Published var useBoldFont: Bool = true
    @Published var fontSmoothing: FontSmoothingChoice = .system
    @Published var showFontPicker: Bool = false

    // Window
    @Published var initialCols: Int = 80
    @Published var initialRows: Int = 24
    @Published var rememberWindowPosition: Bool = true
    @Published var rememberWindowSize: Bool = true
    @Published var showTitleInTab: Bool = true
    @Published var showDirectoryInTitle: Bool = true
    @Published var horizontalPadding: Int = 0
    @Published var verticalPadding: Int = 0

    // Advanced
    @Published var termEnvVar: String = "xterm-256color"
    @Published var enableKittyKeyboard: Bool = false
    @Published var enableKittyGraphics: Bool = true
    @Published var useMetalRendering: Bool = false
    @Published var maxFps: Int = 60
    @Published var configFilePath: String = "~/.config/wezterm/wezterm.lua"
    @Published var logUnknownEscapes: Bool = false
    @Published var enableDebugLogging: Bool = false

    // Available options
    var availableColorSchemes: [String] {
        [
            "Builtin Solarized Dark",
            "Builtin Solarized Light",
            "Builtin Dark",
            "Builtin Light",
            "Dracula",
            "One Dark",
            "Nord",
            "Gruvbox Dark",
            "Tokyo Night",
        ]
    }

    init() {
        loadFromConfig()
    }

    private func loadFromConfig() {
        // Load values from WezTermBridge
        // In real implementation, this would call:
        // let config = WezTermBridge.shared.getCurrentConfig()
        // self.fontFamily = config.fontFamily
        // etc.
    }

    func save() {
        // Save to WezTermBridge
        // In real implementation, this would call:
        // let config = WezTermBridge.shared.getCurrentConfig()
        // config.fontFamily = self.fontFamily
        // WezTermBridge.shared.applyConfig(config)
    }

    func openConfigFile() {
        let path = (configFilePath as NSString).expandingTildeInPath
        NSWorkspace.shared.open(URL(fileURLWithPath: path))
    }

    func reloadConfiguration() {
        // WezTermBridge.shared.reloadConfiguration()
    }

    func resetToDefaults() {
        // Reset all values to defaults
        openWindowOnLaunch = true
        restoreWindows = true
        showTabBar = true
        hideTabBarIfOnlyOneTab = false
        tabBarPosition = .top
        useFancyTabBar = true
        defaultShell = ""
        initialDirectory = ""
        confirmOnQuit = true
        nativeFullScreen = true
        checkForUpdates = true

        colorScheme = "Builtin Solarized Dark"
        adjustForSystemAppearance = true
        windowOpacity = 0.95
        backgroundBlur = 10
        cursorStyle = .block
        cursorBlinks = true
        scrollbackLines = 10000

        fontFamily = "SF Mono"
        fontSize = 13.0
        fontWeight = 400
        lineHeight = 1.1
        cellWidth = 1.0
        useLigatures = false
        useBoldFont = true
        fontSmoothing = .system

        initialCols = 80
        initialRows = 24
        rememberWindowPosition = true
        rememberWindowSize = true
        showTitleInTab = true
        showDirectoryInTitle = true
        horizontalPadding = 0
        verticalPadding = 0

        termEnvVar = "xterm-256color"
        enableKittyKeyboard = false
        enableKittyGraphics = true
        useMetalRendering = false
        maxFps = 60
        logUnknownEscapes = false
        enableDebugLogging = false
    }
}

// MARK: - Supporting Enums

enum TabBarPosition: String {
    case top
    case bottom
}

enum CursorStyleChoice: String {
    case block
    case underline
    case bar
}

enum FontSmoothingChoice: String {
    case system
    case none
    case light
    case medium
    case strong
}
