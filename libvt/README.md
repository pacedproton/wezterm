# LibVT

A high-performance terminal emulation library extracted from [WezTerm](https://wezfurlong.org/wezterm/), designed for integration with VSCode, editors, and other applications.

## Features

- **Complete VT Compatibility**: Full VT100/VT220/xterm support
- **Rich Color Support**: 256-color and true color (24-bit RGB)
- **Modern Protocols**: Mouse events, hyperlinks (OSC 8), image protocols
- **Unicode Support**: Proper width calculation for CJK and emoji
- **Zero Dependencies on GUI**: Pure terminal emulation logic
- **High Performance**: Optimized for speed and low memory usage

## Quick Start

```rust
use libvt::{create_terminal, KeyCode, KeyModifiers};

// Create an 80x24 terminal
let mut term = create_terminal(80, 24);

// Write text with escape sequences
term.write(b"Hello \x1b[1;32mWorld\x1b[0m!\r\n");

// Handle input
let bytes = term.key_down(KeyCode::Enter, KeyModifiers::empty());

// Read screen content
if let Some(cell) = term.get_cell(0, 0) {
    println!("Cell text: {}", cell.text());
}
```

## Use Cases

- **VSCode Integration**: Embed terminal in editors
- **Terminal Multiplexers**: Building tmux/screen alternatives
- **SSH Clients**: Terminal emulation for remote connections
- **Terminal Recording**: Capture and replay terminal sessions
- **WebAssembly**: Run terminal emulation in the browser

## Architecture

LibVT is designed as a standalone library with no GUI dependencies:

- `terminal::Terminal` - Main terminal emulator
- `parser::Parser` - VT escape sequence parser
- `screen::Screen` - Screen buffer management
- `cell::Cell` - Terminal cell representation
- `color::ColorPalette` - Color management
- `input` - Keyboard and mouse input handling

## Safety

This library is written in 100% safe Rust with `#![deny(unsafe_code)]`.

## License

Same as WezTerm (MIT/Apache-2.0 dual-licensed)
