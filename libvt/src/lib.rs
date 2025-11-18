//! `LibVT` - High-Performance Terminal Emulation Library
//!
//! A standalone terminal emulation library extracted from `WezTerm`,
//! designed for integration with `VSCode`, editors, and other applications.
//!
//! # Features
//!
//! - Complete VT100/VT220/xterm compatibility
//! - 256-color and true color support
//! - Mouse protocol support
//! - Sixel/iTerm2/Kitty image protocols
//! - OSC 8 hyperlinks
//! - Unicode support with proper width calculation
//!
//! # Quick Start
//!
//! ```rust
//! use libvt::{Terminal, TerminalConfig};
//!
//! // Create a terminal with default configuration
//! let mut term = Terminal::new(80, 24, TerminalConfig::default());
//!
//! // Write some data
//! term.write(b"Hello, World!\r\n");
//!
//! // Query terminal state
//! let (col, row) = term.cursor_position();
//! println!("Cursor at: ({}, {})", col, row);
//!
//! // Get cell content
//! if let Some(cell) = term.get_cell(0, 0) {
//!     println!("Cell text: {}", cell.text());
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::unused_self)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::struct_excessive_bools)]
#![deny(unsafe_code)]

pub mod cell;
pub mod color;
pub mod cursor;
pub mod error;
pub mod events;
pub mod input;
pub mod parser;
pub mod screen;
pub mod terminal;

pub use cell::{Cell, CellAttributes, Hyperlink, UnderlineStyle};
pub use color::{ColorPalette, ColorSpec, RgbColor};
pub use cursor::{Cursor, CursorShape, CursorVisibility};
pub use error::{Error, Result};
pub use events::{ClipboardType, EventSubscriber, TerminalEvent, TerminalMode};
pub use input::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
pub use screen::{Line, Position, Screen, Selection};
pub use terminal::{Terminal, TerminalConfig, UnicodeVersion};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Create a new terminal instance with default configuration
///
/// # Arguments
///
/// * `cols` - Number of columns (width in characters)
/// * `rows` - Number of rows (height in characters)
///
/// # Example
///
/// ```rust
/// let mut term = libvt::create_terminal(80, 24);
/// term.write(b"Hello");
/// assert_eq!(term.cursor_position(), (5, 0));
/// ```
pub fn create_terminal(cols: u16, rows: u16) -> Terminal {
    Terminal::new(cols, rows, TerminalConfig::default())
}

/// Create a terminal with custom configuration
///
/// # Arguments
///
/// * `cols` - Number of columns
/// * `rows` - Number of rows
/// * `config` - Custom terminal configuration
///
/// # Example
///
/// ```rust
/// use libvt::{TerminalConfig, UnicodeVersion};
///
/// let config = TerminalConfig {
///     scrollback_lines: 50000,
///     enable_images: true,
///     unicode_version: UnicodeVersion::Fifteen,
///     ..Default::default()
/// };
///
/// let mut term = libvt::create_terminal_with_config(120, 40, config);
/// ```
pub fn create_terminal_with_config(cols: u16, rows: u16, config: TerminalConfig) -> Terminal {
    Terminal::new(cols, rows, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_terminal() {
        let term = create_terminal(80, 24);
        assert_eq!(term.size(), (80, 24));
        assert_eq!(term.cursor_position(), (0, 0));
    }

    #[test]
    fn test_simple_write() {
        let mut term = create_terminal(80, 24);
        term.write(b"Hello");
        assert_eq!(term.cursor_position(), (5, 0));
    }

    #[test]
    fn test_newline() {
        let mut term = create_terminal(80, 24);
        term.write(b"Hello\r\nWorld");
        assert_eq!(term.cursor_position(), (5, 1));
    }
}
