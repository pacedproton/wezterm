# WezTerm Codebase Architecture Analysis

## 1. PROJECT OVERVIEW

WezTerm is a sophisticated terminal emulator written in Rust with cross-platform support (Windows, macOS, Linux/Wayland). The codebase is organized as a Cargo workspace with ~50 interconnected crates.

**Key Statistics:**
- Total Project Size: ~600,000+ lines of Rust code
- Main GUI: 177,342 lines (wezterm-gui)
- Window Layer: 22,332 lines (window)
- Terminal Emulation Core: 9,922 lines (term)
- Mux/Multiplexer: 11,689 lines (mux)
- Terminal Utilities: 9,565 lines (termwiz)
- macOS Implementation: 4,695 lines (window/src/os/macos)

---

## 2. DIRECTORY STRUCTURE & ORGANIZATION

```
/home/user/wezterm/
├── Cargo.toml (workspace root)
├── Cargo.lock
│
├── Core Terminal Emulation
│   ├── term/                 # Terminal emulation core (9,922 LOC)
│   ├── termwiz/             # Terminal utilities library (9,565 LOC)
│   ├── wezterm-escape-parser/ # VT escape sequence parsing
│   ├── wezterm-cell/         # Terminal cell data model
│   ├── wezterm-surface/      # Surface and line primitives
│   └── wezterm-term/         # Public terminal emulation API
│
├── GUI & Rendering
│   ├── wezterm-gui/          # Main GUI application (177,342 LOC)
│   │   ├── src/main.rs       # Application entry point
│   │   ├── src/termwindow/   # Terminal window implementation
│   │   ├── src/glyphcache.rs # Font glyph caching
│   │   ├── src/renderstate.rs# Rendering state management
│   │   └── src/termwindow/render/ # Rendering pipeline
│   │
│   └── window/               # Cross-platform window abstraction (22,332 LOC)
│       ├── src/lib.rs        # Window traits and common types
│       ├── src/connection.rs # Window system connection
│       ├── src/os/           # Platform-specific implementations
│       │   ├── windows/      # Windows implementation
│       │   ├── x11/          # X11 implementation
│       │   ├── wayland/      # Wayland implementation
│       │   └── macos/        # macOS/Cocoa implementation (4,695 LOC)
│       │       ├── mod.rs          # Module exports
│       │       ├── app.rs          # NSApplication delegate
│       │       ├── connection.rs   # Connection manager (255 LOC)
│       │       ├── window.rs       # Window implementation (3,440 LOC)
│       │       ├── menu.rs         # Menu system (409 LOC)
│       │       ├── bitmap.rs       # Bitmap rendering
│       │       ├── clipboard.rs    # Clipboard handling
│       │       └── keycodes.rs     # Keycode translation (248 LOC)
│       └── src/egl.rs        # EGL graphics context
│
├── Multiplexing & Sessions
│   ├── mux/                  # Mux core (11,689 LOC)
│   │   ├── src/lib.rs        # Mux structure and notifications
│   │   ├── src/pane.rs       # Pane abstraction
│   │   ├── src/tab.rs        # Tab management
│   │   ├── src/window.rs     # Window management
│   │   ├── src/domain.rs     # Domain abstraction (local, SSH, etc.)
│   │   ├── src/localpane.rs  # Local process pane implementation
│   │   └── src/tmux.rs       # Tmux protocol support
│   │
│   ├── wezterm-mux-server-impl/ # Mux server implementation
│   ├── wezterm-client/           # Mux client
│   └── wezterm-mux-server/       # Mux server binary
│
├── Configuration System
│   ├── config/               # Configuration parsing and defaults
│   │   ├── src/lib.rs        # Config system entry point
│   │   ├── src/config.rs     # Main configuration struct
│   │   ├── src/lua.rs        # Lua configuration support
│   │   ├── src/window.rs     # Window configuration
│   │   ├── src/terminal.rs   # Terminal configuration
│   │   ├── src/keys.rs       # Key binding configuration
│   │   ├── src/keyassignment.rs # Key action assignments
│   │   ├── src/font.rs       # Font configuration
│   │   ├── src/color.rs      # Color scheme configuration
│   │   └── src/daemon.rs     # Daemon configuration
│   └── config/derive/        # Configuration derive macros
│
├── CLI & Entry Points
│   ├── wezterm/              # CLI binary (wezterm command)
│   │   ├── src/main.rs       # CLI entry point
│   │   └── src/cli/          # CLI subcommands (list, split, etc.)
│   └── wezterm-gui-subcommands/ # GUI-related subcommands
│
├── Font & Text Rendering
│   ├── wezterm-font/         # Font loading and shaping
│   │   ├── src/locator/core_text.rs # macOS Core Text integration
│   │   └── src/lib.rs        # Font system
│   ├── deps/freetype/        # FreeType font engine
│   ├── deps/harfbuzz/        # HarfBuzz text shaping
│   ├── deps/fontconfig/      # Fontconfig (Linux)
│   └── deps/cairo/           # Cairo rendering

├── Utilities & Helpers
│   ├── pty/                  # PTY abstraction (portable-pty)
│   ├── wezterm-escape-parser/ # Escape sequence parsing
│   ├── wezterm-input-types/  # Input event types
│   ├── wezterm-dynamic/      # Dynamic configuration
│   ├── wezterm-bidi/         # Bidirectional text support
│   ├── wezterm-char-props/   # Unicode character properties
│   ├── terminfo/             # Terminal capabilities
│   ├── promise/              # Promise/Future utilities
│   ├── filedescriptor/       # File descriptor utilities
│   ├── luahelper/            # Lua scripting support
│   └── [many others]

└── Build & Configuration
    ├── Makefile
    ├── build.rs (various)
    ├── .github/workflows/  # CI/CD pipelines
    └── ci/                 # Build scripts
```

---

## 3. CORE ARCHITECTURAL COMPONENTS

### 3.1 Application Flow Architecture

```
CLI (wezterm/wezterm-gui main.rs)
    │
    └─> Config System (config/)
            │
            └─> Mux (mux/)
                    │
                    ├─> Windows (mux/src/window.rs)
                    ├─> Tabs (mux/src/tab.rs)
                    ├─> Panes (mux/src/pane.rs)
                    │   └─> LocalPane (mux/src/localpane.rs)
                    │           │
                    │           └─> PTY (portable-pty)
                    │
                    └─> Domains (mux/src/domain.rs)
                            ├─> LocalDomain
                            ├─> SshDomain
                            └─> MuxClientDomain

GUI Layer (wezterm-gui/)
    │
    ├─> TermWindow (wezterm-gui/src/termwindow/)
    │       │
    │       ├─> Terminal (term/) [one per pane]
    │       │       │
    │       │       ├─> TerminalState (term/src/terminalstate/)
    │       │       │   ├─> Screen/Scrollback (term/src/screen.rs)
    │       │       │   ├─> Input Handling (term/src/input.rs)
    │       │       │   └─> Mouse/Keyboard (term/src/terminalstate/mouse.rs, keyboard.rs)
    │       │       │
    │       │       └─> Parser (wezterm-escape-parser/)
    │       │           └─> VT Escape Sequences
    │       │
    │       ├─> GlyphCache (wezterm-gui/src/glyphcache.rs)
    │       │   └─> FontConfiguration (wezterm-font/)
    │       │       └─> Platform-specific Font Loading
    │       │           ├─> macOS (Core Text)
    │       │           ├─> Windows (DirectWrite)
    │       │           └─> Linux (Fontconfig)
    │       │
    │       ├─> RenderState (wezterm-gui/src/renderstate.rs)
    │       │   └─> Quad rendering, texture atlases
    │       │
    │       └─> Window (window/)
    │           ├─> macOS (window/src/os/macos/)
    │           ├─> Windows (window/src/os/windows/)
    │           ├─> X11 (window/src/os/x11/)
    │           └─> Wayland (window/src/os/wayland/)
    │
    └─> Rendering Pipeline
            ├─> Input Events -> Pane
            ├─> Pane Output -> Terminal
            ├─> Terminal Rendering -> Glyphs
            ├─> Glyph Layout -> Geometry
            └─> GPU Rendering -> Screen
```

---

## 4. TERMINAL EMULATION CORE (VT PARSING & STATE)

### 4.1 Core Terminal Emulation Stack

**File: `term/src/terminal.rs` (150+ LOC)**
- Main `Terminal` struct - container for terminal state
- Implements terminal size management
- Coordinates between parser and state

**File: `term/src/terminalstate/mod.rs` (Primary implementation)**
- `TerminalState` - the complete terminal state model
  - Cursor position and attributes
  - Screen buffers (primary + alternate)
  - Scrollback management
  - Color palette
  - Tab stops, modes (DECAWM, DECTCEM, etc.)

**Key Submodules in `term/src/terminalstate/`:**
- `performer.rs` - Action performer (executes escape sequences)
- `keyboard.rs` - Keyboard input encoding
- `mouse.rs` - Mouse event handling and encoding
- `image.rs` - Image/Sixel handling
- `iterm.rs` - iTerm2 image protocol
- `kitty.rs` - Kitty image protocol
- `sixel.rs` - Sixel image support

**File: `term/src/screen.rs`**
- Screen buffer model
- Line storage (Surface-based)
- Scrollback management
- Dirty region tracking

### 4.2 Escape Sequence Parsing

**Primary: `wezterm-escape-parser/` (Full escape parsing implementation)**

**Key Files:**
- `src/parser/mod.rs` - Main parser state machine
- `src/csi.rs` - Control Sequence Introducer (CSI) parsing [
- `src/esc.rs` - Escape sequence handling
- `src/osc.rs` - Operating System Command parsing
- `src/apc.rs` - Application Program Command
- `src/parser/sixel.rs` - Sixel image parsing
- `src/color.rs` - Color parsing (256-color, true color)

**Execution Flow:**
```
Raw bytes -> Parser::advance_bytes()
    │
    └─> Tokenizer (recognizes escape patterns)
            │
            └─> CSI/OSC/ESC/APC handlers
                    │
                    └─> Action enum (decoded semantic meaning)
                            │
                            └─> TerminalState::perform_actions()
                                    │
                                    └─> Performer::execute_action()
```

### 4.3 Cell Model & Surface

**File: `wezterm-cell/` (Core cell abstraction)**
- `Cell` - represents a terminal cell (character + attributes)
- Attributes: color, intensity, italic, underline, etc.
- Image support for Sixel/iTerm2
- Hyperlinks (OSC 8)

**File: `wezterm-surface/`**
- `Line` - sequence of cells with bidi/wrapped info
- `Surface` - rectangular grid of lines
- Change tracking for delta updates
- Hyperlink and style information

---

## 5. GUI/WINDOW MANAGEMENT LAYER

### 5.1 Window Abstraction

**File: `window/src/lib.rs` - Cross-platform traits**
```rust
pub trait WindowOps: Send + Sync {
    // Window management
    fn enable_appearance_changed_notifications(&self, ...);
    fn set_title(&self, title: &str);
    fn set_appearance(&self, appearance: Appearance);
    fn get_appearance(&self) -> Appearance;
    fn set_window_state(&self, state: WindowState);
    fn get_window_state(&self) -> WindowState;
    fn set_position(&self, coords: ScreenPoint);
    fn get_position(&self) -> Promise<ScreenPoint>;
    fn set_dimensions(&self, dimensions: Dimensions);
    fn get_dimensions(&self) -> Promise<Dimensions>;
    fn set_window_level(&self, level: WindowLevel);
    fn set_cursor(&self, cursor: MouseCursor);
    fn paint_frame(&self, ...);
    fn invalidate(&self);
}

pub trait Connection: Send + Sync {
    fn new_window(&self, ...) -> Promise<Window>;
    fn default_dpi(&self) -> f64;
    fn get_appearance(&self) -> Appearance;
    fn screens(&self) -> Result<Screens>;
}
```

**File: `window/src/connection.rs`**
- Connection abstraction
- Event loop management
- Screen detection
- DPI handling

### 5.2 Platform-Specific Implementations

**File: `window/src/os/mod.rs` (Platform selection)**
```rust
#[cfg(target_os = "macos")]
pub use self::macos::*;

#[cfg(all(unix, not(target_os = "macos")))]
pub use x_and_wayland::*;

#[cfg(windows)]
pub use self::windows::*;
```

### 5.3 TermWindow - Main GUI Container

**File: `wezterm-gui/src/termwindow/mod.rs`**
- `TermWindow` struct (largest component, ~1000+ lines)
  - Manages all tabs and panes
  - Coordinates input events
  - Manages window state (split panes, tabs)
  - Handles overlay system (copy mode, search, etc.)
  - Configuration updates

**Key Responsibilities:**
1. **Pane Management**
   - Add/remove panes
   - Split panes (horizontal/vertical)
   - Move panes between tabs
   - Tab management

2. **Input Handling**
   - Keyboard events (keyevent.rs)
   - Mouse events (mouseevent.rs)
   - IME composition
   - Key table system

3. **Rendering Coordination**
   - Glyph cache management
   - Shape cache for text
   - RenderState coordination
   - Frame painting

4. **Overlay System**
   - Copy mode overlays
   - Search overlays
   - Character selector
   - Pane selector
   - Confirmation dialogs

### 5.4 Rendering Pipeline

**File: `wezterm-gui/src/glyphcache.rs` (177,342 LOC total)**
- Glyph atlas management
- Font loading and fallback
- Glyph shape caching
- Metrics tracking

**File: `wezterm-gui/src/renderstate.rs`**
- OpenGL/wgpu state management
- Vertex buffer management
- Texture atlas management
- Shader compilation

**File: `wezterm-gui/src/termwindow/render/`**
- `paint.rs` - Main painting logic
- `draw.rs` - Drawing primitives
- `pane.rs` - Pane rendering
- `tab_bar.rs` - Tab bar rendering
- `fancy_tab_bar.rs` - Fancy tab rendering
- `screen_line.rs` - Screen line rendering
- `split.rs` - Pane split rendering

**Rendering Architecture:**
```
TermWindow::paint()
    │
    ├─> RenderState::start_frame()
    │
    ├─> For each pane:
    │   ├─> GlyphCache::shape_line() [for each line]
    │   ├─> Render glyph quads
    │   ├─> Add to vertex buffer
    │   └─> Track dirty regions
    │
    ├─> Render overlays
    ├─> Render tab bar
    ├─> Render scrollbar
    │
    └─> RenderState::finish_frame()
        ├─> Bind textures
        ├─> Set uniforms
        ├─> Draw call(s)
        └─> Present to screen
```

---

## 6. PLATFORM-SPECIFIC CODE (macOS DEEP DIVE)

### 6.1 macOS Implementation Location

**Primary Directory: `window/src/os/macos/` (4,695 LOC)**

### 6.2 Core macOS Files & Their Responsibilities

**1. `window/src/os/macos/connection.rs` (255 LOC)**
```rust
pub struct Connection {
    ns_app: id,  // NSApplication reference
    windows: RefCell<HashMap<usize, Rc<RefCell<WindowInner>>>>,
    next_window_id: AtomicUsize,
    gl_connection: RefCell<Option<Rc<crate::egl::GlConnection>>>,
}
```
- Manages NSApplication lifecycle
- Creates and tracks windows
- Handles screen detection
- Manages GL context for rendering
- Detects system appearance (Light/Dark mode)

**Key Methods:**
- `create_new()` - Initializes NSApplication and sets delegate
- `default_dpi()` - Gets DPI from screen
- `get_appearance()` - Detects Light/Dark/HighContrast mode
- `screens()` - Detects available screens
- `window_by_id()` - Window lookup
- `terminate_message_loop()` - Shutdown handling

**2. `window/src/os/macos/window.rs` (3,440 LOC - largest file)**

**Core Structures:**
```rust
pub struct WindowInner {
    // Window and view references
    ns_window: *mut NSWindow,
    view: id,  // Custom NSView subclass
    
    // Rendering
    gl_context: Option<Rc<crate::egl::GlState>>,
    
    // Input handling
    ime_handler: ImeHandler,
    dead_key_state: DeadKeyStatus,
    
    // State
    last_mouse_coords: Point,
    current_geometry: WindowGeometry,
    screen_changed: bool,
}

pub enum BackendImpl {
    Cgl(Rc<cglbits::GlState>),  // OpenGL on macOS
    Egl(Rc<crate::egl::GlState>), // EGL compatibility
}
```

**Key Responsibilities:**
1. **Window Creation & Management**
   - Creates NSWindow with custom NSView
   - Sets up view hierarchy
   - Manages window decorations (titlebar, etc.)
   - Handles resizing and geometry

2. **Input Handling**
   - Keyboard event translation (keycodes.rs)
   - Mouse event handling
   - IME (Input Method Editor) support
   - Dead key support

3. **Rendering Integration**
   - OpenGL context setup (CGL or EGL)
   - Frame buffer updates
   - Drawable coordinate conversion
   - DPI/scaling handling

4. **Event Handling**
   - Converts NSEvent to window::Event
   - Handles focus changes
   - Monitors appearance changes
   - Tracks window state (fullscreen, minimized, etc.)

**3. `window/src/os/macos/app.rs` (197 LOC)**
- Creates custom NSApplication delegate
- Implements app lifecycle events:
  - `applicationDidBecomeActive`
  - `applicationDidResignActive`
  - `applicationShouldTerminateAfterLastWindowClosed`
  - Application-level event handling

**4. `window/src/os/macos/menu.rs` (409 LOC)**
- Constructs macOS application menu
- File menu (New, Open, Save, Close, Quit)
- Edit menu (Copy, Paste, etc.)
- View menu (Zoom, etc.)
- Application menu (About, Preferences, etc.)
- Keyboard shortcut definitions

**5. `window/src/os/macos/keycodes.rs` (248 LOC)**
- Maps macOS virtual keycodes to cross-platform KeyCode enum
- Handles modifier key state (Cmd, Shift, Alt, Ctrl)
- Converts NSEventModifierFlags

**6. `window/src/os/macos/clipboard.rs` (51 LOC)**
- NSPasteboard integration
- Copy/paste with system clipboard
- Multiple clipboard types support

**7. `window/src/os/macos/bitmap.rs` (60 LOC)**
- Image/bitmap rendering support
- NSBitmapImageRep handling
- Texture creation from images

### 6.3 Graphics & Rendering on macOS

**OpenGL Setup:**
- Primarily uses CGL (Core Graphics Library)
- OpenGL 3.2+ via NSOpenGLContext
- NSOpenGLPixelFormat for context creation
- Supports modern OpenGL features (VBO, VAO, shaders)

**CoreGraphics Integration:**
```rust
extern "C" {
    fn CGSMainConnectionID() -> id;
    fn CGSSetWindowBackgroundBlurRadius(...) -> i32;
}
```
- Accesses window manager (private API)
- Enables window blur effects
- Low-level window manipulation

### 6.4 Font Rendering on macOS

**File: `wezterm-font/src/locator/core_text.rs`**
- Core Text framework integration
- Font enumeration via CTFont APIs
- Glyph shaping with Core Text
- Fallback font chain management

**Font Loading Flow:**
```
FontConfiguration
    │
    └─> core_text::load_fonts()
            │
            ├─> CTFontManager::copy_available_font_families()
            ├─> For each font family:
            │   └─> CTFontCreateWithName()
            └─> Return font objects
```

### 6.5 macOS-Specific Cocoa Dependencies

**From Cargo.toml:**
```toml
[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "=0.25.0"           # Cocoa framework bindings
core-foundation = "=0.10.0" # Core Foundation
core-graphics = "=0.24.0"   # Core Graphics (CG* APIs)
core-text = "=21.0.0"       # Core Text for fonts
objc = "0.2"                # Objective-C runtime
objc2 = "0.6"               # Objective-C 2.0
objc2-core-graphics = "0.3" # Objc2 Core Graphics
plist = "1.7"               # Plist parsing
```

### 6.6 macOS Entry Points for Modification

**For Mac-Native Improvements:**

1. **Metal Rendering** (instead of OpenGL)
   - Replace CGL context in `window/src/os/macos/window.rs`
   - Add Metal rendering backend
   - Implement MTLDevice, MTLCommandQueue

2. **Native macOS UI**
   - Extend `window/src/os/macos/window.rs`
   - Use NSMenu for application menus
   - SwiftUI integration (if desired)
   - Native file dialogs, color pickers

3. **Accessibility**
   - Enhance `window/src/os/macos/window.rs`
   - NSAccessibility protocol implementation
   - Screen reader support

4. **System Integration**
   - Notification Center (already exists in wezterm-toast-notification)
   - Dock integration
   - Spotlight/Finder integration

---

## 7. CONFIGURATION SYSTEM

### 7.1 Configuration Architecture

**File: `config/src/lib.rs`**
- Lazy-loaded global `CONFIG` singleton
- CONFIG_DIRS detection
- Configuration reload watching
- Lua scripting support

**Main Types:**
```rust
pub struct Configuration {
    // Core settings
    leader: KeyCode,
    timeout_milliseconds: u64,
    unix_domains: Vec<UnixDomain>,
    ssh_domains: Vec<SshDomain>,
    
    // UI settings
    window_background_opacity: f32,
    font: Font,
    color_scheme: String,
    colors: Palette,
    
    // Behavior
    mouse_bindings: Vec<MouseBinding>,
    key_bindings: Vec<KeyBinding>,
    key_tables: HashMap<String, Vec<KeyTableEntry>>,
    
    // Terminal
    scrollback_lines: usize,
    bell_animation: BellAnimation,
    window_padding: WindowPadding,
}
```

### 7.2 Configuration File Locations

**File: `config/src/lib.rs`**

Searches in order:
1. User-specified via `--config-file`
2. `$WEZTERM_CONFIG_DIR/wezterm.lua`
3. Linux: `$XDG_CONFIG_HOME/wezterm/wezterm.lua` or `~/.config/wezterm/wezterm.lua`
4. macOS: `~/.config/wezterm/wezterm.lua`
5. Windows: `%APPDATA%\wezterm\wezterm.lua`

### 7.3 Lua Configuration

**File: `config/src/lua.rs`**
- Configuration via Lua scripting
- `mlua` crate for Lua integration
- Available APIs exposed to Lua:
  - Window operations
  - Key handling
  - Terminal control
  - Mux operations
  - Color schemes

**Lua API Crates (lua-api-crates/):**
- `wezterm-mux/` - Mux operations
- `window-funcs/` - Window functions
- `termwiz-funcs/` - Terminal utilities
- `ssh-funcs/` - SSH operations
- `spawn-funcs/` - Spawning
- `plugin/` - Plugin system

### 7.4 Key Configuration Areas

**1. Key Bindings (keys.rs)**
```rust
pub struct KeyBinding {
    pub key: KeyCode,
    pub mods: Modifiers,
    pub action: KeyAssignment,
    pub label: Option<String>,
}
```

**2. Terminal Configuration (terminal.rs)**
- Scrollback lines
- Bell settings
- Font size
- Line height
- Cursor shape/blink

**3. Window Configuration (window.rs)**
- Initial size and position
- Opacity
- Padding
- Decorations
- Title bar style

**4. Color Schemes (color.rs)**
- Palette definitions
- 256-color indices
- ANSI colors
- Background/foreground defaults

---

## 8. BUILD SYSTEM & DEPENDENCIES

### 8.1 Workspace Structure

**Root: `Cargo.toml` (workspace)**

Members include:
- Core: bidi, strip-ansi-escapes, color-types
- Terminal: term, termwiz, wezterm-escape-parser, wezterm-cell, wezterm-surface
- GUI: wezterm-gui, window, wezterm-font
- Mux: mux, wezterm-mux-server, wezterm-client
- Config: config
- CLI: wezterm, wezterm-gui-subcommands
- Utilities: pty, filedescriptor, promise, luahelper, etc.

### 8.2 Platform-Specific Dependencies

**Windows:**
- `winapi` - Win32 API bindings
- `windows` - Modern Windows API (Win32 bindings)
- `embed-resource` - Windows resource embedding

**macOS:**
- `cocoa` - Cocoa framework bindings
- `core-foundation` - Core Foundation bindings
- `core-graphics` - Core Graphics bindings
- `core-text` - Core Text (font handling)
- `objc`, `objc2` - Objective-C runtime

**Linux (X11/Wayland):**
- `x11`, `xcb` - X11 support
- `wayland-client`, `smithay-client-toolkit` - Wayland
- `xkbcommon` - Keyboard handling
- `fontconfig` - Font discovery

### 8.3 Graphics & Rendering Dependencies

- `glium` - OpenGL wrapper
- `wgpu` - WebGPU abstraction (25.0.2)
- `tiny-skia` - Skia subset for drawing
- `image` - Image format support
- `cairo-rs` - Cairo rendering (vendored)

### 8.4 Font Rendering Dependencies

- `freetype` - Font rasterization (vendored)
- `harfbuzz` - Text shaping (vendored)
- `fontconfig` - Font discovery (vendored on some platforms)

### 8.5 Text & Unicode Dependencies

- `finl_unicode` - Unicode utilities
- `wezterm-bidi` - Bidirectional text
- `unicode-segmentation` - Grapheme clusters
- `wezterm-char-props` - Character properties

### 8.6 Build Scripts

**Key build scripts:**
- `wezterm-gui/build.rs` - GUI build (Windows resources, asset embedding)
- `window/build.rs` - Window module build
- `deps/freetype/build.rs` - FreeType compilation
- `deps/harfbuzz/build.rs` - HarfBuzz compilation
- `deps/cairo/build.rs` - Cairo compilation

**Freetype/HarfBuzz/Cairo:**
- Compiled from source via `cc` crate
- Highly portable C code
- Minimal external dependencies

---

## 9. KEY ABSTRACTIONS & INTERFACES

### 9.1 Terminal Model Abstraction

**File: `mux/src/pane.rs`**
```rust
pub trait Pane: Send + Sync {
    fn pane_id(&self) -> PaneId;
    fn render(&self, top: StableRowIndex, ...) -> RenderableDimensions;
    fn perform_actions(&self, actions: Vec<Action>);
    fn key_down(&self, key: KeyCode, mods: Modifiers) -> Result<()>;
    fn mouse_event(&self, event: MouseEvent) -> Result<()>;
    fn writer(&self) -> &dyn Write;
}
```

Implementations:
- `LocalPane` - Local shell process
- `RemotePane` - SSH/Mux remote pane
- `MuxPane` - Lua-scriptable pane

### 9.2 Domain Abstraction

**File: `mux/src/domain.rs`**
```rust
pub trait Domain: Send + Sync {
    fn domain_id(&self) -> DomainId;
    fn domain_name(&self) -> &str;
    fn spawn_pane(&self, opts: SpawnTabDomain) -> Future<(Tab, Pane)>;
    fn attach(&self) -> Future<()>;
}
```

Implementations:
- `LocalDomain` - Local processes
- `SshDomain` - SSH connections
- `MuxClientDomain` - Mux server connections
- `SerialDomain` - Serial connections

### 9.3 Terminal Configuration Abstraction

**File: `term/src/config.rs`**
```rust
pub trait TerminalConfiguration: Send + Sync {
    fn term_program(&self) -> String;
    fn term_program_version(&self) -> String;
    fn unicode_version(&self) -> UnicodeVersion;
    fn color_palette(&self, appearance: Appearance) -> ColorPalette;
}
```

### 9.4 Window Operations Abstraction

**File: `window/src/lib.rs`**

All window operations are trait-based:
- `WindowOps` - Individual window operations
- `Connection` - Connection-level operations
- `EventHandler` - Event callbacks

---

## 10. TERMINAL EMULATION BOUNDARIES

### 10.1 Input Boundary

**Input Sources:**
1. Keyboard events (via window layer)
2. Mouse events (via window layer)
3. Configuration changes (via config system)
4. PTY data (via pty crate)

**Processing:**
```
Event -> TermWindow::handle_event()
  -> Pane::key_down() / mouse_event()
  -> Terminal::key_down() / mouse_event()
  -> Terminal::encode_input()
  -> Write to PTY writer
```

### 10.2 Output Boundary

**Output Sources:**
1. PTY reader (from child process)
2. Data parsed by termwiz escape parser
3. Actions executed on TerminalState

**Processing:**
```
PTY data -> mux::parse_buffered_data()
  -> Parser::advance_bytes() [escape-parser]
  -> Action enum
  -> TerminalState::perform_actions()
  -> Performer::execute_action()
  -> Screen updated
```

### 10.3 Rendering Boundary

**Boundary between Terminal Model and Rendering:**

```
TerminalState::screen [pixels in cells]
  │
  ├─> GlyphCache::shape_line()
  │   [Convert cells to shaped glyphs]
  │
  └─> RenderState::paint()
      [Convert glyphs to GPU geometry]
```

### 10.4 Configuration Boundary

**Configuration is injected at:**
1. Startup (main config file loading)
2. Reload (SIGHUP or explicit reload command)
3. Per-terminal overrides (escape sequences)

**Configuration flows to:**
1. Terminal model (cursor blink, bell, etc.)
2. Window (padding, decorations, etc.)
3. Font system (font family, size, etc.)
4. Color system (palette, appearance)
5. Key bindings (input handling)

---

## 11. KEY FILES FOR MODIFICATION/EXTRACTION

### For Mac-Native Library Extraction

**1. Core Terminal Emulation (Extract as library):**
- `/home/user/wezterm/term/` - Full terminal emulation
- `/home/user/wezterm/wezterm-escape-parser/` - Escape sequence parsing
- `/home/user/wezterm/wezterm-cell/` - Cell model
- `/home/user/wezterm/wezterm-surface/` - Surface model

**Status:** These are already designed as extractable libraries with minimal GUI dependencies.

**2. Font System (Extract as library):**
- `/home/user/wezterm/wezterm-font/` - Font loading and caching
- `/home/user/wezterm/wezterm-font/src/locator/core_text.rs` - macOS font loading
- `/home/user/wezterm/deps/freetype/` - Font rasterization
- `/home/user/wezterm/deps/harfbuzz/` - Text shaping

**3. macOS Window Layer (Replace/enhance):**
- `/home/user/wezterm/window/src/os/macos/window.rs` - Main window implementation (3,440 LOC)
- `/home/user/wezterm/window/src/os/macos/connection.rs` - Connection management (255 LOC)
- `/home/user/wezterm/window/src/os/macos/app.rs` - App delegate (197 LOC)
- `/home/user/wezterm/window/src/os/macos/menu.rs` - Menu system (409 LOC)

**4. GUI Rendering (Replace/enhance):**
- `/home/user/wezterm/wezterm-gui/src/renderstate.rs` - OpenGL state
- `/home/user/wezterm/wezterm-gui/src/glyphcache.rs` - Glyph management
- `/home/user/wezterm/wezterm-gui/src/termwindow/render/` - Rendering pipeline

**For Improvements:**
- Add Metal rendering backend (alternative to OpenGL)
- Enhance macOS app integration (native file dialogs, Spotlight)
- Add SwiftUI/AppKit native UI elements
- Improve accessibility support

### Critical Boundaries to Maintain

1. **Terminal Emulation Core** - Highly stable, well-tested VT100/xterm compatible
2. **Input/Output Encoding** - Critical for terminal compatibility
3. **Cell Model** - Core data structure for all rendering
4. **Configuration System** - Affects all subsystems

### Safe Areas for Modification

1. **Rendering Backend** - Can replace OpenGL with Metal/WebGPU
2. **macOS-Specific Code** - All in `window/src/os/macos/`
3. **Font Rendering** - Can optimize with native APIs
4. **GUI Features** - Layout, tabs, overlays, etc.

---

## 12. SUMMARY OF ARCHITECTURAL STRENGTHS

1. **Excellent Modularization** - Clean separation of terminal emulation, GUI, and platform code
2. **Well-Designed Abstractions** - Trait-based architecture allows for multiple implementations
3. **Extractable Components** - Terminal core, font system, and escape parser are independent crates
4. **Platform Abstraction** - Clear platform-specific boundaries with minimal coupling
5. **Rust Safety** - Memory safety prevents entire categories of bugs
6. **Performance** - Efficient caching (glyph, shape), minimal allocations, GPU rendering

---

## 13. ARCHITECTURAL CONCERNS & CONSIDERATIONS

1. **OpenGL vs Metal** - macOS still uses OpenGL 3.2; Metal would be more modern
2. **Escape Sequence Coverage** - Large feature set means continuous maintenance
3. **Platform Dependencies** - macOS requires Cocoa, Core Text, Core Graphics
4. **Configuration Complexity** - Lua-based configuration has large surface area
5. **Mux Complexity** - Multiplexing layer adds significant complexity

