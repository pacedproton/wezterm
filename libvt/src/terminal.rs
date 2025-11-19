//! Terminal emulator core implementation

use crate::{
    buffer_api::BufferView,
    buffer_set::{BufferId, BufferSet},
    cell::Cell,
    color::ColorPalette,
    cursor::{Cursor, CursorShape},
    events::{EventSubscriber, TerminalEvent},
    input::{KeyCode, KeyModifiers, MouseEvent},
    markers::{DecorationType, Marker, MarkerId, MarkerManager},
    parser::{Action, Parser},
    parser_ext::HandlerRegistry,
    screen::{Line, Position},
    selection::{Selection, SelectionMode},
};
use std::collections::VecDeque;

/// Unicode version for width calculation
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum UnicodeVersion {
    /// Unicode 9.0
    Nine,
    /// Unicode 14.0
    Fourteen,
    /// Unicode 15.0
    #[default]
    Fifteen,
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
///
/// The `Terminal` struct is the core of libvt, managing the complete state
/// of a terminal emulation session including the screen buffer, scrollback,
/// cursor position, and escape sequence parsing.
///
/// # Examples
///
/// ```rust
/// use libvt::{Terminal, TerminalConfig};
///
/// // Create an 80x24 terminal
/// let mut term = Terminal::new(80, 24, TerminalConfig::default());
///
/// // Write text with ANSI escape sequences
/// term.write(b"\x1b[1;31mRed Bold Text\x1b[0m\r\n");
///
/// // Check cursor position
/// let (col, row) = term.cursor_position();
/// assert_eq!(row, 1);
/// ```
pub struct Terminal {
    /// Terminal dimensions
    cols: u16,
    rows: u16,

    /// Buffer set (primary + alternate screens)
    buffers: BufferSet,

    /// Scrollback buffer (only for primary screen)
    scrollback: VecDeque<Line>,

    /// Current scroll position (0 = bottom, positive = scrolled up)
    scroll_position: u32,

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

    /// Selection
    selection: Option<Selection>,

    /// Marker manager for line annotations
    markers: MarkerManager,

    /// Parser extension handlers
    handler_registry: HandlerRegistry,

    /// Bracketed paste mode enabled
    bracketed_paste: bool,
}

impl Terminal {
    /// Create a new terminal
    pub fn new(cols: u16, rows: u16, config: TerminalConfig) -> Self {
        let buffers = BufferSet::new(cols as usize, rows as usize);
        let scrollback = VecDeque::with_capacity(config.scrollback_lines);
        let cursor = Cursor::new_with_shape(config.cursor_shape);

        Self {
            cols,
            rows,
            buffers,
            scrollback,
            scroll_position: 0,
            parser: Parser::new(),
            cursor,
            config,
            events: Vec::new(),
            subscribers: Vec::new(),
            dirty_lines: vec![false; rows as usize],
            title: String::new(),
            working_directory: None,
            selection: None,
            markers: MarkerManager::new(),
            handler_registry: HandlerRegistry::new(),
            bracketed_paste: false,
        }
    }

    /// Write data to the terminal
    ///
    /// This processes the input through the escape sequence parser
    /// and updates terminal state accordingly. Supports standard
    /// VT100/VT220/xterm escape sequences.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw bytes to write (can include escape sequences)
    ///
    /// # Returns
    ///
    /// The number of bytes processed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use libvt::create_terminal;
    ///
    /// let mut term = create_terminal(80, 24);
    ///
    /// // Write plain text
    /// term.write(b"Hello");
    ///
    /// // Write with escape sequences
    /// term.write(b"\x1b[1;32mGreen\x1b[0m");
    ///
    /// // Position cursor and write
    /// term.write(b"\x1b[5;10HAt position");
    /// ```
    pub fn write(&mut self, data: &[u8]) -> usize {
        let actions = self.parser.parse(data);

        for action in actions {
            self.perform_action(action);
        }

        // Emit WriteParsed event after data is successfully parsed
        self.emit_event(TerminalEvent::WriteParsed);

        data.len()
    }

    /// Resize the terminal
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        self.buffers.resize(cols as usize, rows as usize);
        self.dirty_lines = vec![true; rows as usize];
        self.emit_event(TerminalEvent::Resized { cols, rows });
    }

    /// Handle keyboard input, returns bytes to write to PTY
    pub fn key_down(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Vec<u8> {
        let data = self.encode_key(key, modifiers);

        // Emit Data event for user input
        if !data.is_empty() {
            self.emit_event(TerminalEvent::Data(data.clone()));
        }

        data
    }

    /// Send binary data to the terminal (for binary mode)
    pub fn send_binary(&mut self, data: Vec<u8>) {
        self.emit_event(TerminalEvent::Binary(data));
    }

    /// Handle mouse event, returns bytes to write to PTY
    pub fn mouse_event(&mut self, event: MouseEvent) -> Vec<u8> {
        self.encode_mouse(event)
    }

    /// Get a cell at the specified position
    pub fn get_cell(&self, col: u16, row: u16) -> Option<&Cell> {
        self.buffers.active().get_cell(col as usize, row as usize)
    }

    /// Get a mutable cell at the specified position
    pub fn get_cell_mut(&mut self, col: u16, row: u16) -> Option<&mut Cell> {
        let row_usize = row as usize;
        self.mark_dirty(row_usize);
        self.buffers.active_mut().get_cell_mut(col as usize, row_usize)
    }

    /// Get a line at the specified row
    pub fn get_line(&self, row: u16) -> Option<&Line> {
        self.buffers.active().get_line(row as usize)
    }

    /// Get all visible lines
    pub fn visible_lines(&self) -> impl Iterator<Item = &Line> {
        self.buffers.active().lines()
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

    /// Get a read-only view of the active buffer
    ///
    /// Returns a `BufferView` that provides zero-copy access to the terminal buffer,
    /// matching xterm.js's IBuffer interface.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvt::Terminal;
    ///
    /// let mut term = libvt::create_terminal(80, 24);
    /// term.write(b"Hello, World!");
    ///
    /// let buffer = term.get_buffer_view();
    /// let (x, y) = (buffer.cursor_x(), buffer.cursor_y());
    /// println!("Cursor at: ({}, {})", x, y);
    /// ```
    pub fn get_buffer_view(&self) -> BufferView {
        BufferView::new(
            self.buffers.active(),
            self.cursor.col,
            self.cursor.row,
        )
    }

    /// Get a read-only view of the active buffer with scrollback information
    ///
    /// # Arguments
    ///
    /// * `base_y` - The base line offset in scrollback
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvt::Terminal;
    ///
    /// let mut term = libvt::create_terminal(80, 24);
    /// let buffer = term.get_buffer_view_with_scrollback(100);
    /// assert_eq!(buffer.base_y(), 100);
    /// ```
    pub fn get_buffer_view_with_scrollback(&self, base_y: u32) -> BufferView {
        BufferView::with_scrollback(
            self.buffers.active(),
            self.cursor.col,
            self.cursor.row,
            base_y,
        )
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

    /// Get current scroll position (0 = bottom, positive = scrolled up)
    pub fn scroll_position(&self) -> u32 {
        self.scroll_position
    }

    /// Set scroll position
    pub fn scroll(&mut self, position: u32) {
        let max_scroll = self.scrollback.len() as u32;
        self.scroll_position = position.min(max_scroll);
        self.emit_event(TerminalEvent::Scroll {
            position: self.scroll_position,
        });
    }

    /// Scroll up by n lines
    pub fn scroll_up_lines(&mut self, lines: u32) {
        self.scroll(self.scroll_position.saturating_add(lines));
    }

    /// Scroll down by n lines
    pub fn scroll_down_lines(&mut self, lines: u32) {
        self.scroll(self.scroll_position.saturating_sub(lines));
    }

    /// Scroll to top of scrollback
    pub fn scroll_to_top(&mut self) {
        self.scroll(self.scrollback.len() as u32);
    }

    /// Scroll to bottom (normal position)
    pub fn scroll_to_bottom(&mut self) {
        self.scroll(0);
    }

    /// Check if terminal is in alternate screen mode
    pub fn is_alternate_screen(&self) -> bool {
        self.buffers.is_alternate_active()
    }

    /// Switch to alternate screen buffer (used by vim, less, etc.)
    pub fn enter_alternate_screen(&mut self) {
        if self.buffers.switch_to_alternate() {
            // Reset cursor to home position when entering alternate screen
            // (matches behavior of real terminals)
            self.cursor.col = 0;
            self.cursor.row = 0;
            self.emit_event(TerminalEvent::AlternateScreenEnabled);
            self.emit_event(TerminalEvent::BufferChange { alternate: true });
        }
    }

    /// Switch back to primary screen buffer
    pub fn exit_alternate_screen(&mut self) {
        if self.buffers.switch_to_primary() {
            self.emit_event(TerminalEvent::AlternateScreenDisabled);
            self.emit_event(TerminalEvent::BufferChange { alternate: false });
        }
    }

    /// Get the active buffer ID
    pub fn active_buffer(&self) -> BufferId {
        self.buffers.active_buffer_id()
    }

    /// Get selection (if any)
    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }

    /// Set selection with stream mode
    pub fn set_selection(&mut self, start: Position, end: Position) {
        self.selection = Some(Selection::new_stream(start, end));
        self.emit_event(TerminalEvent::SelectionChanged);
    }

    /// Set selection with specific mode
    pub fn set_selection_with_mode(&mut self, start: Position, end: Position, mode: SelectionMode) {
        self.selection = Some(match mode {
            SelectionMode::Stream => Selection::new_stream(start, end),
            SelectionMode::Block => Selection::new_block(start, end),
            SelectionMode::Line => Selection::new_line(start, end),
        });
        self.emit_event(TerminalEvent::SelectionChanged);
    }

    /// Update selection end position (for dragging)
    pub fn update_selection_end(&mut self, end: Position) {
        if let Some(sel) = &mut self.selection {
            sel.update_end(end);
            self.emit_event(TerminalEvent::SelectionChanged);
        }
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.emit_event(TerminalEvent::SelectionChanged);
    }

    /// Get selected text
    pub fn selected_text(&self) -> Option<String> {
        self.selection
            .as_ref()
            .map(|sel| sel.get_text(self.buffers.active()))
    }

    /// Get selected text with trailing whitespace trimmed
    pub fn selected_text_trimmed(&self) -> Option<String> {
        self.selection
            .as_ref()
            .map(|sel| sel.get_text_trimmed(self.buffers.active()))
    }

    // Marker management

    /// Add a marker at the specified line
    ///
    /// Returns the ID of the created marker
    pub fn add_marker(&mut self, line: u32, decoration: DecorationType) -> MarkerId {
        self.markers.add_marker(line, decoration)
    }

    /// Add a marker with a message
    pub fn add_marker_with_message(
        &mut self,
        line: u32,
        decoration: DecorationType,
        message: impl Into<String>,
    ) -> MarkerId {
        self.markers.add_marker_with_message(line, decoration, message)
    }

    /// Remove a marker by ID
    pub fn remove_marker(&mut self, id: MarkerId) -> Option<Marker> {
        self.markers.remove_marker(id)
    }

    /// Get a marker by ID
    pub fn get_marker(&self, id: MarkerId) -> Option<&Marker> {
        self.markers.get_marker(id)
    }

    /// Get all markers for a specific line
    pub fn get_markers_for_line(&self, line: u32) -> Vec<&Marker> {
        self.markers.get_markers_for_line(line)
    }

    /// Get all markers
    pub fn markers(&self) -> impl Iterator<Item = &Marker> {
        self.markers.markers()
    }

    /// Clear all markers
    pub fn clear_markers(&mut self) {
        self.markers.clear();
    }

    // Parser extension API

    /// Get mutable access to the handler registry
    pub fn handler_registry_mut(&mut self) -> &mut HandlerRegistry {
        &mut self.handler_registry
    }

    /// Get access to the handler registry
    pub fn handler_registry(&self) -> &HandlerRegistry {
        &self.handler_registry
    }

    // Bracketed paste mode

    /// Check if bracketed paste mode is enabled
    pub fn is_bracketed_paste_mode(&self) -> bool {
        self.bracketed_paste
    }

    /// Enable or disable bracketed paste mode
    pub fn set_bracketed_paste_mode(&mut self, enabled: bool) {
        self.bracketed_paste = enabled;
    }

    /// Paste text with bracketed paste mode support
    ///
    /// If bracketed paste is enabled, wraps the text with escape sequences
    pub fn paste(&mut self, text: &str) -> Vec<u8> {
        if self.bracketed_paste {
            // ESC [ 200 ~ (start) + text + ESC [ 201 ~ (end)
            let mut result = b"\x1b[200~".to_vec();
            result.extend_from_slice(text.as_bytes());
            result.extend_from_slice(b"\x1b[201~");
            result
        } else {
            text.as_bytes().to_vec()
        }
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
            if let Some(cell) = self.buffers.active_mut().get_cell_mut(col, row) {
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
            0x0A..=0x0C => {
                // Line feed, vertical tab, form feed
                self.emit_event(TerminalEvent::LineFeed);
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
                    self.emit_event(TerminalEvent::WorkingDirectoryChanged(parts[1].to_string()));
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
        // Check if we're on alternate before borrowing mutably
        let is_alternate = self.buffers.is_alternate_active();
        let screen = self.buffers.active_mut();

        // Handle scrolling based on which buffer is active
        if is_alternate {
            // On alternate screen, just remove the top line
            screen.remove_line(0);
        } else {
            // On primary buffer, move top line to scrollback
            if let Some(line) = screen.remove_line(0) {
                if self.scrollback.len() >= self.config.scrollback_lines {
                    self.scrollback.pop_front();
                }
                self.scrollback.push_back(line);
            }
        }

        // Add new empty line at bottom
        screen.push_line(Line::new(self.cols as usize));

        // Update markers (one line scrolled)
        self.markers.handle_scroll(1);

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

    #[test]
    fn test_write_parsed_event() {
        use crate::events::CallbackSubscriber;
        use std::sync::{Arc, Mutex};

        let mut term = Terminal::new(80, 24, TerminalConfig::default());

        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        term.subscribe(Box::new(CallbackSubscriber::new(move |event| {
            events_clone.lock().unwrap().push(event.clone());
        })));

        term.write(b"Hello");

        let recorded = events.lock().unwrap();
        // Should have WriteParsed event
        assert!(recorded
            .iter()
            .any(|e| matches!(e, TerminalEvent::WriteParsed)));
    }

    #[test]
    fn test_linefeed_event() {
        use crate::events::CallbackSubscriber;
        use std::sync::{Arc, Mutex};

        let mut term = Terminal::new(80, 24, TerminalConfig::default());

        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        term.subscribe(Box::new(CallbackSubscriber::new(move |event| {
            events_clone.lock().unwrap().push(event.clone());
        })));

        term.write(b"\n");

        let recorded = events.lock().unwrap();
        // Should have LineFeed event
        assert!(recorded
            .iter()
            .any(|e| matches!(e, TerminalEvent::LineFeed)));
    }

    #[test]
    fn test_scroll_api() {
        let mut term = Terminal::new(80, 24, TerminalConfig::default());

        // Initial position is 0
        assert_eq!(term.scroll_position(), 0);

        // Add some scrollback first by filling the terminal
        for _ in 0..30 {
            term.write(b"Line\n");
        }

        // Scroll to bottom
        term.scroll_to_bottom();
        assert_eq!(term.scroll_position(), 0);

        // Scroll up
        term.scroll_up_lines(5);
        assert_eq!(term.scroll_position(), 5);

        // Scroll down
        term.scroll_down_lines(2);
        assert_eq!(term.scroll_position(), 3);

        // Scroll to bottom
        term.scroll_to_bottom();
        assert_eq!(term.scroll_position(), 0);

        // Scroll to top
        term.scroll_to_top();
        assert!(term.scroll_position() > 0);
    }

    #[test]
    fn test_scroll_event() {
        use crate::events::CallbackSubscriber;
        use std::sync::{Arc, Mutex};

        let mut term = Terminal::new(80, 24, TerminalConfig::default());

        // Add some scrollback first
        for _ in 0..30 {
            term.write(b"Line\n");
        }

        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        term.subscribe(Box::new(CallbackSubscriber::new(move |event| {
            events_clone.lock().unwrap().push(event.clone());
        })));

        term.scroll_up_lines(5);

        let recorded = events.lock().unwrap();
        // Should have Scroll event with position 5
        assert!(recorded.iter().any(|e| matches!(
            e,
            TerminalEvent::Scroll { position: 5 }
        )));
    }

    #[test]
    fn test_data_event() {
        use crate::events::CallbackSubscriber;
        use std::sync::{Arc, Mutex};

        let mut term = Terminal::new(80, 24, TerminalConfig::default());

        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        term.subscribe(Box::new(CallbackSubscriber::new(move |event| {
            events_clone.lock().unwrap().push(event.clone());
        })));

        term.key_down(KeyCode::Char('a'), KeyModifiers::empty());

        let recorded = events.lock().unwrap();
        // Should have Data event with 'a'
        assert!(recorded.iter().any(|e| {
            if let TerminalEvent::Data(data) = e {
                data == &vec![b'a']
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_binary_event() {
        use crate::events::CallbackSubscriber;
        use std::sync::{Arc, Mutex};

        let mut term = Terminal::new(80, 24, TerminalConfig::default());

        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        term.subscribe(Box::new(CallbackSubscriber::new(move |event| {
            events_clone.lock().unwrap().push(event.clone());
        })));

        let binary_data = vec![0x00, 0x01, 0x02, 0xFF];
        term.send_binary(binary_data.clone());

        let recorded = events.lock().unwrap();
        // Should have Binary event
        assert!(recorded.iter().any(|e| {
            if let TerminalEvent::Binary(data) = e {
                data == &binary_data
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_alternate_screen() {
        let mut term = Terminal::new(80, 24, TerminalConfig::default());

        // Initially on primary buffer
        assert!(!term.is_alternate_screen());
        assert_eq!(term.active_buffer(), BufferId::Primary);

        // Write some data to primary
        term.write(b"Primary\n");

        // Switch to alternate
        term.enter_alternate_screen();
        assert!(term.is_alternate_screen());
        assert_eq!(term.active_buffer(), BufferId::Alternate);

        // Alternate should be cleared
        if let Some(cell) = term.get_cell(0, 0) {
            assert_eq!(cell.text(), " ");
        }

        // Write to alternate
        term.write(b"Alternate\n");

        // Switch back to primary
        term.exit_alternate_screen();
        assert!(!term.is_alternate_screen());
        assert_eq!(term.active_buffer(), BufferId::Primary);

        // Primary data should still be there
        if let Some(cell) = term.get_cell(0, 0) {
            assert_eq!(cell.text(), "P");
        }
    }

    #[test]
    fn test_alternate_screen_events() {
        use crate::events::CallbackSubscriber;
        use std::sync::{Arc, Mutex};

        let mut term = Terminal::new(80, 24, TerminalConfig::default());

        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        term.subscribe(Box::new(CallbackSubscriber::new(move |event| {
            events_clone.lock().unwrap().push(event.clone());
        })));

        // Enter alternate screen
        term.enter_alternate_screen();

        let recorded = events.lock().unwrap();
        // Should have both AlternateScreenEnabled and BufferChange events
        assert!(recorded
            .iter()
            .any(|e| matches!(e, TerminalEvent::AlternateScreenEnabled)));
        assert!(recorded
            .iter()
            .any(|e| matches!(e, TerminalEvent::BufferChange { alternate: true })));
    }

    #[test]
    fn test_scrollback_only_on_primary() {
        let mut term = Terminal::new(80, 5, TerminalConfig::default());

        // Fill primary screen (should create scrollback)
        for i in 0..10 {
            term.write(format!("Line {}\n", i).as_bytes());
        }

        // Should have scrollback on primary
        assert!(term.scrollback().len() > 0);
        let primary_scrollback = term.scrollback().len();

        // Switch to alternate
        term.enter_alternate_screen();

        // Fill alternate screen (should NOT add to scrollback)
        for i in 0..10 {
            term.write(format!("Alt {}\n", i).as_bytes());
        }

        // Scrollback should not have grown
        assert_eq!(term.scrollback().len(), primary_scrollback);
    }

    #[test]
    fn test_independent_buffers() {
        let mut term = Terminal::new(80, 24, TerminalConfig::default());

        // Write to primary
        term.write(b"Primary");
        if let Some(cell) = term.get_cell(0, 0) {
            assert_eq!(cell.text(), "P");
        }

        // Switch to alternate and write
        term.enter_alternate_screen();
        term.write(b"Alternate");
        if let Some(cell) = term.get_cell(0, 0) {
            assert_eq!(cell.text(), "A");
        }

        // Switch back to primary
        term.exit_alternate_screen();
        if let Some(cell) = term.get_cell(0, 0) {
            assert_eq!(cell.text(), "P");
        }

        // Switch to alternate again
        term.enter_alternate_screen();
        // Should be cleared (not the previous "Alternate" text)
        if let Some(cell) = term.get_cell(0, 0) {
            assert_eq!(cell.text(), " ");
        }
    }
}
