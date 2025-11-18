# CLAUDE.md - WezTerm Codebase Guide for AI Assistants

This document provides a comprehensive guide to the WezTerm codebase structure, development workflows, and key conventions for AI assistants working with this project.

## Table of Contents

1. [Project Overview](#project-overview)
2. [Repository Architecture](#repository-architecture)
3. [Key Crates and Components](#key-crates-and-components)
4. [Development Workflows](#development-workflows)
5. [Build System](#build-system)
6. [Testing Guidelines](#testing-guidelines)
7. [Code Conventions](#code-conventions)
8. [Component Interactions](#component-interactions)
9. [Common Tasks](#common-tasks)
10. [Important Files and Locations](#important-files-and-locations)

---

## Project Overview

**WezTerm** is a GPU-accelerated cross-platform terminal emulator and multiplexer written in Rust by @wez.

- **Language**: Rust (Edition 2018)
- **Architecture**: Monorepo with Cargo Workspace (46 crates)
- **Platforms**: Linux (X11/Wayland), macOS, Windows
- **Documentation**: https://wezterm.org/
- **Repository**: https://github.com/wezterm/wezterm

### Core Features
- Terminal emulation with VT100+ compatibility
- Built-in multiplexer (alternative to tmux/screen)
- GPU-accelerated rendering (wgpu/glium)
- Lua-based configuration system
- SSH client integration
- Cross-platform support with native windowing

---

## Repository Architecture

### Structure Type
**Layered Monorepo** with clear separation of concerns using Cargo workspace.

### Root Configuration Files
- `Cargo.toml` - Workspace manifest (19 workspace members, 46 total crates)
- `Cargo.lock` - Dependency lock file
- `.rustfmt.toml` - Code formatting rules
- `deny.toml` - Dependency vetting via cargo-deny
- `Makefile` - Build targets and common commands
- `get-deps` - Script to install system dependencies

### Directory Organization

```
wezterm/
├── Core Terminal Components
│   ├── term/                      # Terminal state machine (VT100+ emulator)
│   ├── vtparse/                   # Low-level escape sequence parser
│   ├── wezterm-escape-parser/     # High-level escape parser (Sixel, Kitty, iTerm2)
│   ├── wezterm-cell/              # Terminal cell data structure
│   ├── wezterm-surface/           # Grid of cells for terminal content
│   └── termwiz/                   # Terminal UI library (standalone)
│
├── Multiplexing System
│   ├── mux/                       # Core multiplexing engine (Window/Tab/Pane/Domain)
│   ├── pty/                       # PTY abstraction (Unix/Windows)
│   └── wezterm-blob-leases/       # Blob lease management
│
├── Rendering Pipeline
│   ├── window/                    # Cross-platform windowing (X11/Wayland/Windows/macOS)
│   ├── wezterm-font/              # Font rendering and shaping (FreeType/HarfBuzz)
│   ├── color-types/               # Color type definitions
│   └── wezterm-char-props/        # Unicode character properties
│
├── Configuration System
│   ├── config/                    # Lua configuration system (mlua integration)
│   ├── wezterm-dynamic/           # Dynamic value system for config serialization
│   └── lua-api-crates/            # 15+ Lua API modules
│       ├── mux-lua/               # Mux API bindings
│       ├── window-funcs/          # Window control
│       ├── spawn-funcs/           # Process spawning
│       ├── ssh-funcs/             # SSH functions
│       ├── filesystem/            # File operations
│       ├── color-funcs/           # Color manipulation
│       └── ... (10+ more)
│
├── Networking & Communication
│   ├── codec/                     # Binary protocol encoding (leb128, zstd)
│   ├── wezterm-ssh/               # SSH support (libssh2-rs wrapper)
│   ├── wezterm-client/            # Client-side multiplexer connection
│   ├── wezterm-mux-server-impl/   # Server-side multiplexer protocol
│   ├── wezterm-uds/               # Unix domain socket utilities
│   └── async_ossl/                # Async OpenSSL support
│
├── Binary Targets
│   ├── wezterm/                   # CLI client for remote control
│   ├── wezterm-gui/               # Main GUI terminal application
│   ├── wezterm-mux-server/        # Multiplexer server daemon
│   └── wezterm-gui-subcommands/   # Shared CLI subcommands
│
├── Utility Crates
│   ├── base91/                    # Base91 encoding
│   ├── bidi/                      # Bidirectional text support (Hebrew, Arabic)
│   ├── bintree/                   # Binary tree data structure
│   ├── filedescriptor/            # File descriptor utilities
│   ├── frecency/                  # Frecency scoring algorithm
│   ├── lfucache/                  # LFU cache implementation
│   ├── promise/                   # Promise/Future utilities
│   ├── rangeset/                  # Range set data structure
│   ├── ratelim/                   # Rate limiting
│   ├── strip-ansi-escapes/        # ANSI escape sequence stripper
│   ├── tabout/                    # Table output formatting
│   ├── umask/                     # Umask utilities
│   └── procinfo/                  # Process information
│
├── Vendored Dependencies
│   └── deps/
│       ├── cairo/                 # Vector graphics library
│       ├── fontconfig/            # Font configuration
│       ├── freetype/              # Font rendering
│       └── harfbuzz/              # Text shaping
│
├── Documentation & Assets
│   ├── docs/                      # Documentation (MkDocs format)
│   ├── assets/                    # UI assets, fonts, icons, shell integration
│   ├── test-data/                 # Test fixtures
│   └── ci/                        # CI scripts
│
└── Build & CI
    ├── .github/workflows/         # 40+ CI/CD workflows
    └── nix/                       # Nix package support
```

---

## Key Crates and Components

### Binary Targets (Executables)

| Crate | Location | Purpose | Entry Point |
|-------|----------|---------|-------------|
| **wezterm** | `/wezterm/` | CLI client for remote control | `wezterm/src/main.rs` |
| **wezterm-gui** | `/wezterm-gui/` | Main GUI terminal application | `wezterm-gui/src/main.rs` |
| **wezterm-mux-server** | `/wezterm-mux-server/` | Multiplexer server daemon | `wezterm-mux-server/src/main.rs` |
| **strip-ansi-escapes** | `/strip-ansi-escapes/` | Utility to strip ANSI escapes | `strip-ansi-escapes/src/main.rs` |

### Core Domain Crates

#### Terminal Emulation Layer
- **term** (wezterm-term): VT100+ terminal state machine. Handles escape sequences, cursor positioning, screen buffer management. Compatible with xterm behavior.
- **vtparse**: Low-level VT escape sequence parser (CSI, OSC, DCS, etc.)
- **wezterm-escape-parser**: High-level parsing for modern protocols (Sixel, Kitty graphics, iTerm2 inline images)
- **wezterm-cell**: Terminal cell representation (character + attributes + hyperlinks)
- **wezterm-surface**: 2D grid of cells representing terminal content

#### Multiplexing Layer
- **mux**: Core multiplexing engine providing Window/Tab/Pane/Domain abstractions
  - Trait-based design allows multiple pane types (LocalPane, ClientPane, SshPane, TmuxPane)
  - Central event notification via `MuxNotification` enum
  - Activity tracking and workspace management
- **pty** (portable-pty): Cross-platform PTY creation and management

#### Rendering Layer
- **window**: Cross-platform windowing abstraction (X11, Wayland, Windows, macOS)
- **wezterm-font**: Font loading, rendering, and shaping (FreeType + HarfBuzz)
- **wezterm-gui/src/**: Main rendering coordination
  - `glyphcache.rs`: LFU cache of rendered glyphs
  - `renderstate.rs`: GPU render state management
  - `shader.wgsl`: WGPU shaders (modern path)
  - `glyph-*.glsl`: Glium shaders (legacy path)

#### Configuration Layer
- **config**: Lua-based configuration system using mlua (Lua 5.4)
- **wezterm-dynamic**: Dynamic JSON-like value system for config serialization
- **lua-api-crates/**: 15+ feature-specific Lua API modules

#### Networking Layer
- **codec**: Binary protocol for client-server communication (leb128, zstd compression)
- **wezterm-ssh**: SSH implementation (libssh2 wrapper)
- **wezterm-client**: Client-side mux server connection
- **wezterm-mux-server-impl**: Server-side mux protocol handler

---

## Development Workflows

### Setting Up Development Environment

1. **Install System Dependencies**:
   ```bash
   ./get-deps
   # or for testing/docs:
   ./get-deps --testing --docs
   ```

2. **Verify Installation**:
   ```bash
   rustc --version
   cargo --version
   ```

### Iteration and Development

#### Type Checking (Fast)
```bash
cargo check                    # Check all code
cargo check -p wezterm-gui    # Check specific crate
```

#### Building

```bash
# Development build (debug mode, fast compilation)
cargo build

# Release build (optimized)
cargo build --release

# Build specific binaries
cargo build -p wezterm
cargo build -p wezterm-gui
cargo build -p wezterm-mux-server

# Use Makefile for convenience
make build
make check
```

#### Running

```bash
# Run in debug mode (more detailed backtraces)
cargo run

# Run specific binary
cargo run -p wezterm-gui

# With backtrace
RUST_BACKTRACE=1 cargo run

# Run with specific configuration
cargo run -- start --config-file /path/to/config.lua
```

### Code Quality

#### Formatting
```bash
# Format all code (requires nightly for some features)
cargo +nightly fmt
# or
make fmt

# Check formatting without changing files
cargo +nightly fmt -- --check
```

**Formatting Rules** (from `.rustfmt.toml`):
- Edition: 2018
- Tab spaces: 4
- Imports granularity: Module
- Keep entries alphabetically ordered

#### Linting
```bash
cargo clippy --all-targets --all-features
```

### Debugging

```bash
# Build debug version
cargo build

# Debug with gdb
gdb ./target/debug/wezterm
# In gdb:
(gdb) break rust_panic    # Tab completion for symbol
(gdb) run
(gdb) bt                  # Backtrace
```

---

## Build System

### Cargo Workspace

The project uses **Cargo Workspace** with resolver version 2 for workspace-aware dependency resolution.

**Workspace members** (19 explicit + others via path dependencies):
- See `Cargo.toml` [workspace.members] section

### Build Profiles

**Release Profile** (`Cargo.toml`):
```toml
[profile.release]
opt-level = 3
```

**Development Profile**:
- Default settings
- Optional: `split-debuginfo = "unpacked"` (disabled on Windows)

### Feature Flags

#### GUI-Specific Features
- `vendored-fonts` (default) - Bundle fonts for consistent experience
- `vendor-nerd-font-symbols` - Bundle Nerd Font symbols
- `vendor-jetbrains` - Bundle JetBrains Mono font
- `vendor-roboto` - Bundle Roboto font
- `vendor-noto-emoji` - Bundle Noto Color Emoji
- `wayland` (default on Linux) - Wayland support
- `dhat-heap` - Heap profiling

#### Protocol Features
- `use_serde` - Serialization for network transport
- `use_image` - Image protocol support (Sixel, Kitty, iTerm2)
- `tmux_cc` - Tmux control mode support
- `kitty-shm` - Kitty shared memory protocol

#### Distro Features
- `distro-defaults` - Recommended for distribution packages
  - Sets `check_for_updates` to false
  - See `README-DISTRO-MAINTAINER.md` for details

### Makefile Targets

```bash
make all          # Same as make build
make build        # Build main binaries
make check        # Run cargo check on key crates
make test         # Run tests with nextest
make fmt          # Format code with nightly rustfmt
make docs         # Build documentation
make servedocs    # Serve docs locally with auto-reload
```

### CI/CD

**Location**: `.github/workflows/`

**Key Workflows**:
- `fmt.yml` - Code formatting check (runs on every PR)
- `gen_*_continuous.yml` - Continuous builds for various platforms
- `gen_*_tag.yml` - Release builds on git tags
- `nix*.yml` - NixOS package support
- `termwiz.yml` - Termwiz crate specific tests
- `wezterm_ssh.yml` - SSH functionality tests
- `pages.yml` - Documentation deployment

**Supported Platforms**:
- CentOS 9
- Debian 11, 12
- Fedora 39, 40, 41
- Ubuntu 20.04, 22.04, 24.04
- macOS
- Windows

---

## Testing Guidelines

### Running Tests

```bash
# All tests
cargo test --all

# Specific crate
cargo test -p mux
cargo test -p term

# With nextest (faster parallel execution)
cargo nextest run

# Doc tests
cargo test --doc

# Benchmarks
cargo bench -p termwiz
```

### Test Organization

**Unit Tests**: Located in source files (`#[cfg(test)] mod tests { ... }`)

**Integration Tests**: Located in `tests/` subdirectories of crates
- `bidi/tests/conformance.rs` - Bidirectional text conformance
- `wezterm-dynamic/tests/` - Dynamic value serialization tests
- `wezterm-ssh/tests/` - SSH protocol tests (e2e, sftp)

**Test Infrastructure**:
- **k9** - Snapshot testing for terminal state
- **criterion** - Benchmarking
- **rstest** - Parameterized tests
- **predicates** - Assertion helpers
- **assert_fs** - File system assertions

**Terminal State Tests**: Special comprehensive suite in `term/src/test/`
- `test/mod.rs` - 105 KB comprehensive VTE behavior tests
- `test/performer.rs` - Abstract test executor
- Tests cover: Sixel, Kitty, iTerm2, keyboard, mouse, images

### Writing Tests

**From CONTRIBUTING.md**:
- Please include tests to cover your changes
- Use helper classes for terminal behavior tests
- Add comments to clarify test intent
- Example test structure:
  ```rust
  #[test]
  fn test_terminal_behavior() {
      // Setup terminal state
      let mut term = Terminal::new(...);

      // Perform action
      term.perform(Action::Print('A'));

      // Assert expected state using k9
      k9::snapshot!(term.screen().visible_rows(), "expected_output");
  }
  ```

---

## Code Conventions

### Rust Style

**From `.rustfmt.toml` and CONTRIBUTING.md**:
- Edition: 2018
- Indentation: 4 spaces (no tabs)
- Imports: Module-level granularity
- Keep imports alphabetically ordered
- **Always run `cargo +nightly fmt` before submitting**

### Terminal Emulation Compatibility

**From CONTRIBUTING.md**:
- Aim for xterm compatibility when adding terminal escape sequence support
- Reference: https://invisible-island.net/xterm/ctlseqs/ctlseqs.html
- Compatibility with existing terminal behavior is prioritized

### Documentation

**From CONTRIBUTING.md**:
- Include documentation for new features or behavior changes
- Don't worry about perfect English; capturing intent is most important
- Add comments to tests to clarify purpose
- Documentation lives in `docs/` directory (MkDocs format)

### Code Location Guidelines

**From CONTRIBUTING.md**:
- **term/** - Terminal model code (escape sequences, terminal state)
- **wezterm-gui/src/** - GUI rendering and interaction
- Both are agnostic of each other's implementation details

### Commit Messages

- Be descriptive about what changed and why
- Reference issue numbers when applicable
- See recent commits for style examples:
  ```
  cargo update
  docs: changelog for #7366
  max_fps config fix
  ```

### Dependency Management

- Use workspace dependencies defined in root `Cargo.toml`
- Vendored critical dependencies in `deps/` for reproducibility
- Use `cargo deny` for license and security vetting
- Keep dependencies up to date but test thoroughly

---

## Component Interactions

### High-Level Architecture Flow

```
User Input → GUI (wezterm-gui) → Mux → Domain → Pane → Terminal (term)
                                                         ↓
                                                    VT Parser
                                                         ↓
                                                    Cell Buffer
                                                         ↓
                                            Font Shaping + GPU Rendering
                                                         ↓
                                                    Display
```

### Data Flow for Terminal Rendering

1. **Input**: User keystrokes captured by window event loop
2. **Routing**: Input → Mux → Active Domain → Active Pane
3. **Terminal Processing**: Pane → Terminal state machine (term crate)
4. **Parsing**: Escape sequences → vtparse/wezterm-escape-parser
5. **State Update**: Terminal state machine updates cell buffer (wezterm-surface)
6. **Rendering Preparation**:
   - Font shaping with HarfBuzz (wezterm-font)
   - Glyph caching (glyphcache.rs)
   - Quad mesh generation
7. **GPU Rendering**: WGPU or Glium shader execution
8. **Display**: Window manager update

### Client-Server Communication Flow

```
CLI Client (wezterm) or GUI Client (wezterm-gui)
    ↓
wezterm-client (discovery, domain API, pane proxy)
    ↓
codec (serialize with leb128/zstd)
    ↓
Transport (Unix socket, TCP, SSH)
    ↓
wezterm-mux-server-impl (session handler, dispatch)
    ↓
Mux Core (Windows/Tabs/Panes)
```

### Configuration System Flow

1. Find `wezterm.lua` in standard locations
2. Load via mlua (Lua 5.4 embedded)
3. Execute Lua script with full API access (lua-api-crates)
4. Validate against schema (wezterm-config-derive)
5. Create `ConfigHandle`
6. Watch for changes (via `notify` crate)
7. Reload and notify GUI on changes

---

## Common Tasks

### Adding a New Escape Sequence

1. **Identify the sequence** in xterm documentation
2. **Add parsing** in `vtparse/` or `wezterm-escape-parser/`
3. **Implement behavior** in `term/src/terminalstate/mod.rs`
4. **Add tests** in `term/src/test/`
5. **Document** in `docs/escape-sequences.md`

Example locations:
- Parser: `term/src/terminalstate/performer.rs`
- Tests: `term/src/test/mod.rs`

### Adding a New Configuration Option

1. **Define in config struct**: `config/src/lib.rs`
2. **Add derive macro**: Use `#[serde(default)]` for optional fields
3. **Expose to Lua**: Add to config schema
4. **Document**: Add to `docs/config/` directory
5. **Use in code**: Access via `config.your_option`

### Adding a New Lua API Function

1. **Choose appropriate lua-api-crate** or create new one
2. **Implement function**: Use `mlua` bindings
3. **Register in module**: Add to Lua module exports
4. **Document**: Add to relevant docs page
5. **Add tests**: Test Lua integration

### Modifying GUI Rendering

Key files:
- `wezterm-gui/src/termwindow/mod.rs` - Main rendering coordination
- `wezterm-gui/src/glyphcache.rs` - Glyph caching
- `wezterm-gui/src/renderstate.rs` - GPU state
- `wezterm-gui/src/shader.wgsl` - WGPU shaders

### Working with the Multiplexer

Key files:
- `mux/src/lib.rs` - Main mux interface
- `mux/src/pane.rs` - Pane trait definition
- `mux/src/domain.rs` - Domain abstraction
- `mux/src/activity.rs` - Notification system

### Debugging Terminal Behavior

1. **Enable logging**:
   ```bash
   WEZTERM_LOG=trace wezterm start
   ```

2. **Use escape sequence debugging**:
   ```bash
   wezterm start -- cat test-file
   ```

3. **Examine terminal state**:
   - Add debug prints in `term/src/terminalstate/performer.rs`
   - Check cell buffer in `wezterm-surface`

4. **Use snapshot tests**:
   ```rust
   k9::snapshot!(term.screen().visible_rows(), "test_name");
   ```

---

## Important Files and Locations

### Configuration and Setup
- `/Cargo.toml` - Workspace manifest
- `/.rustfmt.toml` - Formatting rules
- `/deny.toml` - Dependency vetting configuration
- `/get-deps` - System dependency installer script

### Documentation
- `/README.md` - Project overview
- `/CONTRIBUTING.md` - Contribution guidelines
- `/README-DISTRO-MAINTAINER.md` - Distribution packaging notes
- `/docs/` - Full documentation (MkDocs)
- `/docs/config/` - Configuration reference

### Build and CI
- `/Makefile` - Common build targets
- `/.github/workflows/` - CI/CD workflows
- `/ci/` - CI scripts
- `/ci/build-docs.sh` - Documentation build script

### Main Entry Points
- `/wezterm/src/main.rs` - CLI client entry
- `/wezterm-gui/src/main.rs` - GUI application entry
- `/wezterm-mux-server/src/main.rs` - Server daemon entry

### Core Implementation
- `/term/src/lib.rs` - Terminal emulator core
- `/term/src/terminalstate/mod.rs` - Terminal state machine
- `/mux/src/lib.rs` - Multiplexer core
- `/config/src/lib.rs` - Configuration system
- `/window/src/` - Windowing system

### Rendering
- `/wezterm-gui/src/termwindow/mod.rs` - Main window (2500+ lines)
- `/wezterm-gui/src/glyphcache.rs` - Glyph caching
- `/wezterm-font/src/` - Font system
- `/wezterm-gui/src/shader.wgsl` - GPU shaders

### Utilities
- `/wezterm-dynamic/src/` - Dynamic value system
- `/codec/src/` - Protocol encoding
- `/vtparse/src/` - Escape sequence parser

### Tests
- `/term/src/test/` - Terminal emulation tests
- `/test-data/` - Test fixtures
- Various `tests/` subdirectories in crates

---

## Additional Resources

### External Documentation
- **User Documentation**: https://wezterm.org/
- **xterm Control Sequences**: https://invisible-island.net/xterm/ctlseqs/ctlseqs.html
- **Rust Documentation**: https://doc.rust-lang.org/
- **mlua Documentation**: https://docs.rs/mlua/

### Getting Help
- **GitHub Issues**: https://github.com/wezterm/wezterm/issues
- **GitHub Discussions**: https://github.com/wezterm/wezterm/discussions
- **Matrix Room**: https://app.element.io/#/room/#wezterm:matrix.org

### Key Dependencies to Understand
- **mlua** (0.9) - Lua 5.4 embedding
- **wgpu** (25.0) - Modern GPU API
- **smol** (2.0) - Async runtime
- **serde** (1.0) - Serialization framework
- **harfbuzz** - Text shaping
- **freetype** - Font rendering

---

## AI Assistant Guidelines

When working with this codebase:

1. **Understand the layer**: Identify which architectural layer you're working in (terminal, mux, rendering, config, etc.)

2. **Follow the conventions**: Always run `cargo +nightly fmt` before suggesting code changes

3. **Test your changes**: Include appropriate tests using k9 for terminal behavior, rstest for parameterized tests

4. **Check compatibility**: For terminal emulation, verify xterm compatibility

5. **Document your work**: Add comments and update relevant documentation

6. **Consider all platforms**: Remember this is cross-platform (Linux/macOS/Windows)

7. **Use workspace dependencies**: Check root `Cargo.toml` for shared dependencies

8. **Respect the architecture**: Don't introduce dependencies between layers that shouldn't interact

9. **Performance matters**: This is a GUI application; consider rendering performance

10. **Look at existing patterns**: Search for similar implementations before creating new patterns

---

**Last Updated**: 2025-11-18
**WezTerm Version**: As of commit 118802c

For questions or clarifications about this guide, refer to CONTRIBUTING.md or open a discussion on GitHub.
