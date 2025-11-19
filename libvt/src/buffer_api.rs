//! Buffer Access API for zero-copy terminal buffer access
//!
//! Provides interfaces matching xterm.js IBuffer, IBufferLine, and IBufferCell
//! for efficient access to terminal buffer contents.

use crate::{
    cell::Cell,
    color::ColorSpec,
    screen::{Line, Screen},
};

/// Buffer access interface (xterm.js: IBuffer)
///
/// Provides read-only access to a terminal buffer (primary or alternate).
pub struct BufferView<'a> {
    screen: &'a Screen,
    cursor_y: u16,
    cursor_x: u16,
    base_y: u32,
    view_port_y: u32,
}

impl<'a> BufferView<'a> {
    /// Create a new buffer view
    pub fn new(screen: &'a Screen, cursor_x: u16, cursor_y: u16) -> Self {
        Self {
            screen,
            cursor_x,
            cursor_y,
            base_y: 0,
            view_port_y: 0,
        }
    }

    /// Create with scrollback offset
    pub fn with_scrollback(
        screen: &'a Screen,
        cursor_x: u16,
        cursor_y: u16,
        base_y: u32,
    ) -> Self {
        Self {
            screen,
            cursor_x,
            cursor_y,
            base_y,
            view_port_y: 0,
        }
    }

    /// Get buffer type (always returns false for now, as we handle alternate elsewhere)
    pub fn is_alternate_buffer(&self) -> bool {
        false
    }

    /// Get the y position of the cursor (0-indexed)
    pub fn cursor_y(&self) -> u16 {
        self.cursor_y
    }

    /// Get the x position of the cursor (0-indexed)
    pub fn cursor_x(&self) -> u16 {
        self.cursor_x
    }

    /// Get the line position of the cursor relative to the buffer
    pub fn base_y(&self) -> u32 {
        self.base_y
    }

    /// Get the viewport's current scroll position
    pub fn view_port_y(&self) -> u32 {
        self.view_port_y
    }

    /// Get a line from the buffer
    pub fn get_line(&self, y: usize) -> Option<BufferLineView> {
        self.screen.get_line(y).map(BufferLineView::new)
    }

    /// Get the number of lines in the buffer
    pub fn length(&self) -> usize {
        self.screen.height()
    }
}

/// Buffer line access interface (xterm.js: IBufferLine)
///
/// Provides read-only access to a single line in the terminal buffer.
pub struct BufferLineView<'a> {
    line: &'a Line,
}

impl<'a> BufferLineView<'a> {
    /// Create a new buffer line view
    pub fn new(line: &'a Line) -> Self {
        Self { line }
    }

    /// Get the length of the line (number of cells)
    pub fn length(&self) -> usize {
        self.line.width()
    }

    /// Check if the line is wrapped
    pub fn is_wrapped(&self) -> bool {
        self.line.is_wrapped()
    }

    /// Get a cell from the line
    pub fn get_cell(&self, x: usize) -> Option<BufferCellView> {
        self.line.get_cell(x).map(|cell| BufferCellView {
            cell,
            position: x,
        })
    }

    /// Get the trimmed length (excluding trailing whitespace)
    pub fn get_trimmed_length(&self) -> usize {
        let mut len = self.line.width();
        while len > 0 {
            if let Some(cell) = self.line.get_cell(len - 1) {
                if cell.text().trim().is_empty() {
                    len -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        len
    }

    /// Translate the line to a string
    pub fn translate_to_string(&self, trim_right: bool, start: usize, end: usize) -> String {
        let start = start.min(self.line.width());
        let end = if trim_right {
            self.get_trimmed_length()
        } else {
            end.min(self.line.width())
        };

        self.line.get_text_range(start, end)
    }
}

/// Buffer cell access interface (xterm.js: IBufferCell)
///
/// Provides read-only access to a single cell in the terminal buffer.
#[derive(Clone)]
pub struct BufferCellView<'a> {
    cell: &'a Cell,
    position: usize,
}

impl<'a> BufferCellView<'a> {
    /// Get the character(s) in the cell
    pub fn get_chars(&self) -> &str {
        self.cell.text()
    }

    /// Get the width of the cell (1 for normal, 2 for wide characters, 0 for combining)
    pub fn get_width(&self) -> usize {
        self.cell.width() as usize
    }

    /// Get the character code point
    pub fn get_code(&self) -> u32 {
        self.cell
            .text()
            .chars()
            .next()
            .map(|c| c as u32)
            .unwrap_or(0)
    }

    /// Get foreground color
    pub fn get_fg_color(&self) -> u32 {
        match &self.cell.attrs().foreground {
            ColorSpec::Default => 0xFF_FF_FF, // White default
            ColorSpec::Ansi(idx) | ColorSpec::Palette(idx) => *idx as u32,
            ColorSpec::Rgb(rgb) => {
                ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | (rgb.b as u32)
            }
        }
    }

    /// Get background color
    pub fn get_bg_color(&self) -> u32 {
        match &self.cell.attrs().background {
            ColorSpec::Default => 0x00_00_00, // Black default
            ColorSpec::Ansi(idx) | ColorSpec::Palette(idx) => *idx as u32,
            ColorSpec::Rgb(rgb) => {
                ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | (rgb.b as u32)
            }
        }
    }

    /// Check if cell has bold attribute
    pub fn is_bold(&self) -> bool {
        self.cell.attrs().bold
    }

    /// Check if cell has italic attribute
    pub fn is_italic(&self) -> bool {
        self.cell.attrs().italic
    }

    /// Check if cell has underline attribute
    pub fn is_underline(&self) -> bool {
        !matches!(self.cell.attrs().underline, crate::cell::UnderlineStyle::None)
    }

    /// Check if cell has blink attribute
    pub fn is_blink(&self) -> bool {
        self.cell.attrs().blink
    }

    /// Check if cell has inverse attribute
    pub fn is_inverse(&self) -> bool {
        self.cell.attrs().reverse
    }

    /// Check if cell is invisible
    pub fn is_invisible(&self) -> bool {
        self.cell.attrs().hidden
    }

    /// Check if cell has strikethrough attribute
    pub fn is_strikethrough(&self) -> bool {
        self.cell.attrs().strikethrough
    }

    /// Check if cell has dim attribute
    pub fn is_dim(&self) -> bool {
        self.cell.attrs().dim
    }

    /// Get position in line
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get the underlying cell
    pub fn cell(&self) -> &Cell {
        self.cell
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::Screen;

    fn create_test_screen() -> Screen {
        let mut screen = Screen::new(10, 5);

        // Fill with test data
        for (row, text) in ["Hello", "World", "Test", "Data", "Line5"]
            .iter()
            .enumerate()
        {
            for (col, ch) in text.chars().enumerate() {
                if let Some(cell) = screen.get_cell_mut(col, row) {
                    cell.set_text(ch.to_string());
                }
            }
        }

        screen
    }

    #[test]
    fn test_buffer_view_creation() {
        let screen = create_test_screen();
        let view = BufferView::new(&screen, 5, 2);

        assert_eq!(view.cursor_x(), 5);
        assert_eq!(view.cursor_y(), 2);
        assert_eq!(view.base_y(), 0);
        assert_eq!(view.view_port_y(), 0);
    }

    #[test]
    fn test_buffer_view_length() {
        let screen = create_test_screen();
        let view = BufferView::new(&screen, 0, 0);

        assert_eq!(view.length(), 5);
    }

    #[test]
    fn test_buffer_view_get_line() {
        let screen = create_test_screen();
        let view = BufferView::new(&screen, 0, 0);

        let line = view.get_line(0);
        assert!(line.is_some());

        let line = view.get_line(100);
        assert!(line.is_none());
    }

    #[test]
    fn test_buffer_line_view_length() {
        let screen = create_test_screen();
        let view = BufferView::new(&screen, 0, 0);
        let line = view.get_line(0).unwrap();

        assert_eq!(line.length(), 10);
    }

    #[test]
    fn test_buffer_line_view_get_cell() {
        let screen = create_test_screen();
        let view = BufferView::new(&screen, 0, 0);
        let line = view.get_line(0).unwrap();

        let cell = line.get_cell(0);
        assert!(cell.is_some());
        assert_eq!(cell.unwrap().get_chars(), "H");

        let cell = line.get_cell(1);
        assert!(cell.is_some());
        assert_eq!(cell.unwrap().get_chars(), "e");
    }

    #[test]
    fn test_buffer_line_translate_to_string() {
        let screen = create_test_screen();
        let view = BufferView::new(&screen, 0, 0);
        let line = view.get_line(0).unwrap();

        let text = line.translate_to_string(false, 0, 5);
        assert_eq!(text, "Hello");

        let text = line.translate_to_string(false, 1, 4);
        assert_eq!(text, "ell");
    }

    #[test]
    fn test_buffer_cell_view_chars() {
        let screen = create_test_screen();
        let view = BufferView::new(&screen, 0, 0);
        let line = view.get_line(0).unwrap();
        let cell = line.get_cell(0).unwrap();

        assert_eq!(cell.get_chars(), "H");
        assert_eq!(cell.get_code(), 'H' as u32);
    }

    #[test]
    fn test_buffer_cell_view_width() {
        let screen = create_test_screen();
        let view = BufferView::new(&screen, 0, 0);
        let line = view.get_line(0).unwrap();
        let cell = line.get_cell(0).unwrap();

        assert_eq!(cell.get_width(), 1);
    }

    #[test]
    fn test_buffer_cell_view_colors() {
        let screen = create_test_screen();
        let view = BufferView::new(&screen, 0, 0);
        let line = view.get_line(0).unwrap();
        let cell = line.get_cell(0).unwrap();

        // Default colors
        assert_eq!(cell.get_fg_color(), 0xFF_FF_FF);
        assert_eq!(cell.get_bg_color(), 0x00_00_00);
    }

    #[test]
    fn test_buffer_cell_view_attributes() {
        let screen = create_test_screen();
        let view = BufferView::new(&screen, 0, 0);
        let line = view.get_line(0).unwrap();
        let cell = line.get_cell(0).unwrap();

        // Default attributes (all false)
        assert!(!cell.is_bold());
        assert!(!cell.is_italic());
        assert!(!cell.is_underline());
        assert!(!cell.is_blink());
        assert!(!cell.is_inverse());
        assert!(!cell.is_invisible());
        assert!(!cell.is_strikethrough());
        assert!(!cell.is_dim());
    }

    #[test]
    fn test_buffer_cell_view_position() {
        let screen = create_test_screen();
        let view = BufferView::new(&screen, 0, 0);
        let line = view.get_line(0).unwrap();

        let cell = line.get_cell(0).unwrap();
        assert_eq!(cell.position(), 0);

        let cell = line.get_cell(3).unwrap();
        assert_eq!(cell.position(), 3);
    }

    #[test]
    fn test_buffer_line_get_trimmed_length() {
        let screen = create_test_screen();
        let view = BufferView::new(&screen, 0, 0);
        let line = view.get_line(0).unwrap();

        // "Hello     " should trim to 5
        let trimmed = line.get_trimmed_length();
        assert_eq!(trimmed, 5);
    }

    #[test]
    fn test_buffer_view_with_scrollback() {
        let screen = create_test_screen();
        let view = BufferView::with_scrollback(&screen, 3, 2, 100);

        assert_eq!(view.cursor_x(), 3);
        assert_eq!(view.cursor_y(), 2);
        assert_eq!(view.base_y(), 100);
    }

    #[test]
    fn test_buffer_line_wrapped() {
        let mut screen = Screen::new(10, 5);

        // Set wrapped flag
        if let Some(line) = screen.get_line_mut(0) {
            line.set_wrapped(true);
        }

        let view = BufferView::new(&screen, 0, 0);
        let line = view.get_line(0).unwrap();

        assert!(line.is_wrapped());
    }
}
