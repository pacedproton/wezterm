//! Node.js bindings for libvt terminal emulation library
//!
//! This module provides NAPI-RS bindings to expose libvt functionality
//! to Node.js/TypeScript applications.

#![deny(clippy::all)]

use napi::{bindgen_prelude::*, JsFunction};
use napi_derive::napi;
use std::sync::{Arc, Mutex};

/// Terminal configuration for Node.js
#[napi(object)]
#[derive(Debug, Clone)]
pub struct TerminalConfig {
    /// Number of scrollback lines
    pub scrollback_lines: Option<u32>,
    /// Enable image protocol support
    pub enable_images: Option<bool>,
    /// Enable hyperlinks
    pub enable_hyperlinks: Option<bool>,
    /// Enable bracketed paste
    pub bracketed_paste: Option<bool>,
}

impl From<TerminalConfig> for libvt::TerminalConfig {
    fn from(config: TerminalConfig) -> Self {
        let mut term_config = libvt::TerminalConfig::default();

        if let Some(lines) = config.scrollback_lines {
            term_config.scrollback_lines = lines as usize;
        }
        if let Some(images) = config.enable_images {
            term_config.enable_images = images;
        }
        if let Some(links) = config.enable_hyperlinks {
            term_config.enable_hyperlinks = links;
        }
        if let Some(paste) = config.bracketed_paste {
            term_config.bracketed_paste = paste;
        }

        term_config
    }
}

/// Cell attributes for Node.js
#[napi(object)]
#[derive(Debug, Clone)]
pub struct CellAttributes {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub reverse: bool,
    pub dim: bool,
    pub hidden: bool,
    pub blink: bool,
}

/// Terminal cell for Node.js
#[napi(object)]
#[derive(Debug, Clone)]
pub struct Cell {
    pub text: String,
    pub fg_color: u32,
    pub bg_color: u32,
    pub attrs: CellAttributes,
}

/// Cursor position
#[napi(object)]
#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    pub col: u16,
    pub row: u16,
}

/// Terminal emulator instance
#[napi]
pub struct Terminal {
    inner: Arc<Mutex<libvt::Terminal>>,
}

#[napi]
impl Terminal {
    /// Create a new terminal
    #[napi(constructor)]
    pub fn new(cols: u16, rows: u16, config: Option<TerminalConfig>) -> Result<Self> {
        let term_config = config
            .map(libvt::TerminalConfig::from)
            .unwrap_or_default();

        Ok(Self {
            inner: Arc::new(Mutex::new(libvt::Terminal::new(
                cols,
                rows,
                term_config,
            ))),
        })
    }

    /// Write data to the terminal
    #[napi]
    pub fn write(&self, data: Buffer) -> Result<()> {
        let mut term = self
            .inner
            .lock()
            .map_err(|e| Error::from_reason(format!("Lock error: {e}")))?;

        term.write(&data);
        Ok(())
    }

    /// Get cursor position
    #[napi]
    pub fn cursor_position(&self) -> Result<CursorPosition> {
        let term = self
            .inner
            .lock()
            .map_err(|e| Error::from_reason(format!("Lock error: {e}")))?;

        let (col, row) = term.cursor_position();
        Ok(CursorPosition { col, row })
    }

    /// Get terminal size
    #[napi]
    pub fn size(&self) -> Result<CursorPosition> {
        let term = self
            .inner
            .lock()
            .map_err(|e| Error::from_reason(format!("Lock error: {e}")))?;

        let (cols, rows) = term.size();
        Ok(CursorPosition {
            col: cols,
            row: rows,
        })
    }

    /// Resize the terminal
    #[napi]
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let mut term = self
            .inner
            .lock()
            .map_err(|e| Error::from_reason(format!("Lock error: {e}")))?;

        term.resize(cols, rows);
        Ok(())
    }

    /// Get a cell at the specified position
    #[napi]
    pub fn get_cell(&self, col: u16, row: u16) -> Result<Option<Cell>> {
        let term = self
            .inner
            .lock()
            .map_err(|e| Error::from_reason(format!("Lock error: {e}")))?;

        let cell = term.get_cell(col, row);

        Ok(cell.map(|c| {
            let attrs = c.attrs();

            Cell {
                text: c.text().to_string(),
                fg_color: match attrs.foreground {
                    libvt::ColorSpec::Default => 0xFF_FF_FF,
                    libvt::ColorSpec::Ansi(idx) | libvt::ColorSpec::Palette(idx) => idx as u32,
                    libvt::ColorSpec::Rgb(rgb) => {
                        ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | (rgb.b as u32)
                    }
                },
                bg_color: match attrs.background {
                    libvt::ColorSpec::Default => 0x00_00_00,
                    libvt::ColorSpec::Ansi(idx) | libvt::ColorSpec::Palette(idx) => idx as u32,
                    libvt::ColorSpec::Rgb(rgb) => {
                        ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | (rgb.b as u32)
                    }
                },
                attrs: CellAttributes {
                    bold: attrs.bold,
                    italic: attrs.italic,
                    underline: !matches!(attrs.underline, libvt::UnderlineStyle::None),
                    strikethrough: attrs.strikethrough,
                    reverse: attrs.reverse,
                    dim: attrs.dim,
                    hidden: attrs.hidden,
                    blink: attrs.blink,
                },
            }
        }))
    }

    /// Get text from a line
    #[napi]
    pub fn get_line_text(&self, row: u16) -> Result<Option<String>> {
        let term = self
            .inner
            .lock()
            .map_err(|e| Error::from_reason(format!("Lock error: {e}")))?;

        Ok(term.get_line(row).map(|line| line.to_string()))
    }

    /// Get all visible text
    #[napi]
    pub fn get_visible_text(&self) -> Result<String> {
        let term = self
            .inner
            .lock()
            .map_err(|e| Error::from_reason(format!("Lock error: {e}")))?;

        let lines: Vec<String> = term.visible_lines().map(|line| line.to_string()).collect();

        Ok(lines.join("\n"))
    }

    /// Clear the selection
    #[napi]
    pub fn clear_selection(&self) -> Result<()> {
        let mut term = self
            .inner
            .lock()
            .map_err(|e| Error::from_reason(format!("Lock error: {e}")))?;

        term.clear_selection();
        Ok(())
    }

    /// Clear all markers
    #[napi]
    pub fn clear_markers(&self) -> Result<()> {
        let mut term = self
            .inner
            .lock()
            .map_err(|e| Error::from_reason(format!("Lock error: {e}")))?;

        term.clear_markers();
        Ok(())
    }

    /// Check if a line is in the dirty set
    #[napi]
    pub fn is_line_dirty(&self, row: u16) -> Result<bool> {
        let term = self
            .inner
            .lock()
            .map_err(|e| Error::from_reason(format!("Lock error: {e}")))?;

        Ok(term
            .dirty_lines()
            .get(row as usize)
            .copied()
            .unwrap_or(false))
    }

    /// Clear dirty tracking
    #[napi]
    pub fn clear_dirty(&self) -> Result<()> {
        let mut term = self
            .inner
            .lock()
            .map_err(|e| Error::from_reason(format!("Lock error: {e}")))?;

        term.clear_dirty();
        Ok(())
    }
}

/// Get library version
#[napi]
pub fn version() -> String {
    libvt::VERSION.to_string()
}

/// Create a simple terminal for testing
#[napi]
pub fn create_terminal(cols: u16, rows: u16) -> Result<Terminal> {
    Terminal::new(cols, rows, None)
}
