//! Terminal emulator core implementation

use crate::{
    cell::Cell,
    color::ColorPalette,
    cursor::{Cursor, CursorShape},
    events::{EventSubscriber, TerminalEvent},
    input::{KeyCode, KeyModifiers, MouseEvent},
    parser::{Action, Parser},
    screen::{Line, Position, Screen, Selection},
};
use std::collections::VecDeque;

/// Unicode version for width calculation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnicodeVersion {
    /// Unicode 9.0
    Nine,
    /// Unicode 14.0
    Fourteen,
    /// Unicode 15.0
    Fifteen,
}

impl Default for UnicodeVersion {
    fn default() -> Self {
        Self::Fifteen
    }
}

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
            unicode_version: UnicodeVersion::default(),
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

    /// Terminal title
    title: String,

    /// Working directory
    working_directory: Option<String>,

    /// Alternate screen active
    alternate_screen: bool,

    /// Selection
    selection: Option<Selection>,
}

impl Terminal {
    /// Create a new terminal
    pub fn new(cols: u16, rows: u16, config: TerminalConfig) -> Self {
        let screen = Screen::new(cols as usize, rows as usize);
        let scrollback = VecDeque::with_capacity(config.scrollback_lines);
        let cursor = Cursor::new_with_shape(config.cursor_shape);

        Self {
            cols,
            rows,
            screen,
            scrollback,
            parser: Parser::new(),
            cursor,
            config,
            events: Vec::new(),
            subscribers: Vec::new(),
            dirty_lines: vec![false; rows as usize],
            title: String::new(),
            working_directory: None,
            alternate_screen: false,
            selection: None,
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

    /// Handle keyboard input, returns bytes to write to PTY
    pub fn key_down(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Vec<u8> {
        self.encode_key(key, modifiers)
    }

    /// Handle mouse event, returns bytes to write to PTY
    pub fn mouse_event(&mut self, event: MouseEvent) -> Vec<u8> {
        self.encode_mouse(event)
    }

    /// Get a cell at the specified position
    pub fn get_cell(&self, col: u16, row: u16) -> Option<&Cell> {
        self.screen.get_cell(col as usize, row as usize)
    }

    /// Get a mutable cell at the specified position
    pub fn get_cell_mut(&mut self, col: u16, row: u16) -> Option<&mut Cell> {
        let row_usize = row as usize;
        self.mark_dirty(row_usize);
        self.screen.get_cell_mut(col as usize, row_usize)
    }

    /// Get a line at the specified row
    pub fn get_line(&self, row: u16) -> Option<&Line> {
        self.screen.get_line(row as usize)
    }

    /// Get all visible lines
    pub fn visible_lines(&self) -> impl Iterator<Item = &Line> {
        self.screen.lines()
    }

    /// Get current cursor position (0-indexed)
    pub fn cursor_position(&self) -> (u16, u16) {
        (self.cursor.col, self.cursor.row)
    }

    /// Get cursor state
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// Get mutable cursor state
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
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

    /// Mark a specific line as dirty
    fn mark_dirty(&mut self, row: usize) {
        if row < self.dirty_lines.len() {
            self.dirty_lines[row] = true;
        }
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
        &self.title
    }

    /// Get current working directory (set via OSC 7)
    pub fn working_directory(&self) -> Option<&str> {
        self.working_directory.as_deref()
    }

    /// Check if terminal is in alternate screen mode
    pub fn is_alternate_screen(&self) -> bool {
        self.alternate_screen
    }

    /// Get selection (if any)
    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }

    /// Set selection
    pub fn set_selection(&mut self, start: Position, end: Position) {
        self.selection = Some(Selection { start, end });
        self.emit_event(TerminalEvent::SelectionChanged);
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.emit_event(TerminalEvent::SelectionChanged);
    }

    /// Get selected text
    pub fn selected_text(&self) -> Option<String> {
        self.selection.as_ref().map(|sel| {
            self.screen.get_text_in_range(sel.start, sel.end)
        })
    }

    /// Get terminal configuration
    pub fn config(&self) -> &TerminalConfig {
        &self.config
    }

    /// Update terminal configuration
    pub fn set_config(&mut self, config: TerminalConfig) {
        self.config = config;
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

    fn print(&mut self, c: char) {
        let col = self.cursor.col as usize;
        let row = self.cursor.row as usize;

        if col < self.cols as usize {
            if let Some(cell) = self.screen.get_cell_mut(col, row) {
                cell.set_text(c.to_string());
                cell.set_attrs(self.cursor.attrs.clone());
            }
            self.cursor.col += 1;
            self.mark_dirty(row);
        }

        // Handle line wrap
        if self.cursor.col >= self.cols {
            self.cursor.col = 0;
            self.cursor.row += 1;

            if self.cursor.row >= self.rows {
                self.scroll_up();
                self.cursor.row = self.rows - 1;
            }
        }
    }

    fn control(&mut self, ctrl: u8) {
        match ctrl {
            0x07 => {
                // Bell
                self.emit_event(TerminalEvent::Bell);
            }
            0x08 => {
                // Backspace
                if self.cursor.col > 0 {
                    self.cursor.col -= 1;
                }
            }
            0x09 => {
                // Tab
                let tab_stop = ((self.cursor.col / 8) + 1) * 8;
                self.cursor.col = tab_stop.min(self.cols - 1);
            }
            0x0A | 0x0B | 0x0C => {
                // Line feed, vertical tab, form feed
                self.cursor.row += 1;
                if self.cursor.row >= self.rows {
                    self.scroll_up();
                    self.cursor.row = self.rows - 1;
                }
            }
            0x0D => {
                // Carriage return
                self.cursor.col = 0;
            }
            _ => {}
        }
    }

    fn csi(&mut self, _csi: Vec<u8>) {
        // TODO: Implement CSI sequence handling
        // This is where cursor movement, colors, etc. are handled
    }

    fn esc(&mut self, _esc: Vec<u8>) {
        // TODO: Implement ESC sequence handling
    }

    fn osc(&mut self, osc: Vec<u8>) {
        // Parse OSC sequence
        let s = String::from_utf8_lossy(&osc);
        let parts: Vec<&str> = s.splitn(2, ';').collect();

        if parts.len() >= 2 {
            match parts[0] {
                "0" | "2" => {
                    // Set title
                    self.title = parts[1].to_string();
                    self.emit_event(TerminalEvent::TitleChanged(self.title.clone()));
                }
                "7" => {
                    // Set working directory
                    self.working_directory = Some(parts[1].to_string());
                    self.emit_event(TerminalEvent::WorkingDirectoryChanged(
                        parts[1].to_string(),
                    ));
                }
                _ => {}
            }
        }
    }

    fn dcs(&mut self, _dcs: Vec<u8>) {
        // TODO: Implement DCS sequence handling (Sixel, etc.)
    }

    fn apc(&mut self, _apc: Vec<u8>) {
        // TODO: Implement APC sequence handling (Kitty images, etc.)
    }

    fn scroll_up(&mut self) {
        // Move top line to scrollback
        if let Some(line) = self.screen.remove_line(0) {
            if self.scrollback.len() >= self.config.scrollback_lines {
                self.scrollback.pop_front();
            }
            self.scrollback.push_back(line);
        }

        // Add new empty line at bottom
        self.screen.push_line(Line::new(self.cols as usize));

        // Mark all lines as dirty
        for i in 0..self.dirty_lines.len() {
            self.dirty_lines[i] = true;
        }
    }

    fn emit_event(&mut self, event: TerminalEvent) {
        self.events.push(event.clone());

        for subscriber in &self.subscribers {
            subscriber.on_event(&event);
        }
    }

    fn encode_key(&self, key: KeyCode, modifiers: KeyModifiers) -> Vec<u8> {
        // Simple key encoding - real implementation would be more complex
        match key {
            KeyCode::Char(c) => {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+key encoding
                    let ctrl_char = (c as u8).wrapping_sub(b'a' - 1);
                    vec![ctrl_char]
                } else {
                    c.to_string().into_bytes()
                }
            }
            KeyCode::Enter => vec![0x0D],
            KeyCode::Tab => vec![0x09],
            KeyCode::Backspace => vec![0x7F],
            KeyCode::Escape => vec![0x1B],
            KeyCode::Up => b"\x1b[A".to_vec(),
            KeyCode::Down => b"\x1b[B".to_vec(),
            KeyCode::Right => b"\x1b[C".to_vec(),
            KeyCode::Left => b"\x1b[D".to_vec(),
            KeyCode::Home => b"\x1b[H".to_vec(),
            KeyCode::End => b"\x1b[F".to_vec(),
            KeyCode::PageUp => b"\x1b[5~".to_vec(),
            KeyCode::PageDown => b"\x1b[6~".to_vec(),
            KeyCode::Delete => b"\x1b[3~".to_vec(),
            KeyCode::Insert => b"\x1b[2~".to_vec(),
            _ => vec![],
        }
    }

    fn encode_mouse(&self, _event: MouseEvent) -> Vec<u8> {
        // TODO: Implement mouse encoding based on enabled protocols
        // (SGR, URXVT, X10, etc.)
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_creation() {
        let term = Terminal::new(80, 24, TerminalConfig::default());
        assert_eq!(term.size(), (80, 24));
        assert_eq!(term.cursor_position(), (0, 0));
    }

    #[test]
    fn test_simple_print() {
        let mut term = Terminal::new(80, 24, TerminalConfig::default());
        term.write(b"A");
        assert_eq!(term.cursor_position(), (1, 0));
    }

    #[test]
    fn test_carriage_return() {
        let mut term = Terminal::new(80, 24, TerminalConfig::default());
        term.write(b"Hello\r");
        assert_eq!(term.cursor_position(), (0, 0));
    }

    #[test]
    fn test_line_feed() {
        let mut term = Terminal::new(80, 24, TerminalConfig::default());
        term.write(b"Hello\n");
        assert_eq!(term.cursor_position(), (5, 1));
    }

    #[test]
    fn test_crlf() {
        let mut term = Terminal::new(80, 24, TerminalConfig::default());
        term.write(b"Hello\r\nWorld");
        assert_eq!(term.cursor_position(), (5, 1));
    }

    #[test]
    fn test_bell_event() {
        let mut term = Terminal::new(80, 24, TerminalConfig::default());
        term.write(b"\x07");
        let events = term.take_events();
        assert!(events.iter().any(|e| matches!(e, TerminalEvent::Bell)));
    }

    #[test]
    fn test_resize() {
        let mut term = Terminal::new(80, 24, TerminalConfig::default());
        term.resize(120, 40);
        assert_eq!(term.size(), (120, 40));
    }

    #[test]
    fn test_key_encoding() {
        let term = Terminal::new(80, 24, TerminalConfig::default());

        let enter = term.encode_key(KeyCode::Enter, KeyModifiers::empty());
        assert_eq!(enter, vec![0x0D]);

        let up = term.encode_key(KeyCode::Up, KeyModifiers::empty());
        assert_eq!(up, b"\x1b[A".to_vec());
    }
}
