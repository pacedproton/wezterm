//! Cursor state and management

use crate::cell::CellAttributes;

/// Cursor shape
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorShape {
    /// Block cursor (default)
    #[default]
    Block,
    /// Underline cursor
    Underline,
    /// Bar/beam cursor
    Bar,
    /// Hidden cursor
    Hidden,
}

/// Cursor visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorVisibility {
    /// Cursor is visible
    #[default]
    Visible,
    /// Cursor is hidden
    Hidden,
}

/// Cursor state
#[derive(Debug, Clone)]
pub struct Cursor {
    /// Column position (0-indexed)
    pub col: u16,
    /// Row position (0-indexed)
    pub row: u16,
    /// Cursor shape
    pub shape: CursorShape,
    /// Cursor visibility
    pub visibility: CursorVisibility,
    /// Whether cursor is blinking
    pub blinking: bool,
    /// Current cell attributes (for new text)
    pub attrs: CellAttributes,
    /// Saved cursor position (for DECSC/DECRC)
    saved_position: Option<(u16, u16)>,
    /// Saved attributes
    saved_attrs: Option<CellAttributes>,
}

impl Cursor {
    /// Create a new cursor at position (0, 0)
    pub fn new() -> Self {
        Self {
            col: 0,
            row: 0,
            shape: CursorShape::default(),
            visibility: CursorVisibility::default(),
            blinking: true,
            attrs: CellAttributes::default(),
            saved_position: None,
            saved_attrs: None,
        }
    }

    /// Create a new cursor with a specific shape
    pub fn new_with_shape(shape: CursorShape) -> Self {
        Self {
            col: 0,
            row: 0,
            shape,
            visibility: CursorVisibility::default(),
            blinking: true,
            attrs: CellAttributes::default(),
            saved_position: None,
            saved_attrs: None,
        }
    }

    /// Move cursor to absolute position
    pub fn move_to(&mut self, col: u16, row: u16) {
        self.col = col;
        self.row = row;
    }

    /// Move cursor relative to current position
    pub fn move_relative(&mut self, cols: i16, rows: i16) {
        self.col = (self.col as i16 + cols).max(0) as u16;
        self.row = (self.row as i16 + rows).max(0) as u16;
    }

    /// Move cursor up
    pub fn move_up(&mut self, n: u16) {
        self.row = self.row.saturating_sub(n);
    }

    /// Move cursor down
    pub fn move_down(&mut self, n: u16) {
        self.row = self.row.saturating_add(n);
    }

    /// Move cursor left
    pub fn move_left(&mut self, n: u16) {
        self.col = self.col.saturating_sub(n);
    }

    /// Move cursor right
    pub fn move_right(&mut self, n: u16) {
        self.col = self.col.saturating_add(n);
    }

    /// Move to beginning of line
    pub fn carriage_return(&mut self) {
        self.col = 0;
    }

    /// Move to beginning of next line
    pub fn new_line(&mut self) {
        self.col = 0;
        self.row = self.row.saturating_add(1);
    }

    /// Save cursor position and attributes (DECSC)
    pub fn save(&mut self) {
        self.saved_position = Some((self.col, self.row));
        self.saved_attrs = Some(self.attrs.clone());
    }

    /// Restore cursor position and attributes (DECRC)
    pub fn restore(&mut self) {
        if let Some((col, row)) = self.saved_position {
            self.col = col;
            self.row = row;
        }
        if let Some(attrs) = self.saved_attrs.take() {
            self.attrs = attrs;
        }
    }

    /// Show cursor
    pub fn show(&mut self) {
        self.visibility = CursorVisibility::Visible;
    }

    /// Hide cursor
    pub fn hide(&mut self) {
        self.visibility = CursorVisibility::Hidden;
    }

    /// Toggle cursor visibility
    pub fn toggle_visibility(&mut self) {
        self.visibility = match self.visibility {
            CursorVisibility::Visible => CursorVisibility::Hidden,
            CursorVisibility::Hidden => CursorVisibility::Visible,
        };
    }

    /// Check if cursor is visible
    pub fn is_visible(&self) -> bool {
        matches!(self.visibility, CursorVisibility::Visible)
    }

    /// Set cursor shape
    pub fn set_shape(&mut self, shape: CursorShape) {
        self.shape = shape;
    }

    /// Set cursor blinking
    pub fn set_blinking(&mut self, blinking: bool) {
        self.blinking = blinking;
    }

    /// Reset cursor to default state
    pub fn reset(&mut self) {
        self.col = 0;
        self.row = 0;
        self.shape = CursorShape::default();
        self.visibility = CursorVisibility::default();
        self.blinking = true;
        self.attrs = CellAttributes::default();
        self.saved_position = None;
        self.saved_attrs = None;
    }

    /// Get position as tuple
    pub fn position(&self) -> (u16, u16) {
        (self.col, self.row)
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_creation() {
        let cursor = Cursor::new();
        assert_eq!(cursor.position(), (0, 0));
        assert_eq!(cursor.shape, CursorShape::Block);
        assert!(cursor.is_visible());
    }

    #[test]
    fn test_cursor_movement() {
        let mut cursor = Cursor::new();

        cursor.move_to(10, 5);
        assert_eq!(cursor.position(), (10, 5));

        cursor.move_up(2);
        assert_eq!(cursor.position(), (10, 3));

        cursor.move_down(1);
        assert_eq!(cursor.position(), (10, 4));

        cursor.move_left(3);
        assert_eq!(cursor.position(), (7, 4));

        cursor.move_right(5);
        assert_eq!(cursor.position(), (12, 4));
    }

    #[test]
    fn test_cursor_save_restore() {
        let mut cursor = Cursor::new();
        cursor.move_to(10, 20);
        cursor.attrs.set_bold(true);

        cursor.save();
        cursor.move_to(0, 0);
        cursor.attrs.set_bold(false);

        cursor.restore();
        assert_eq!(cursor.position(), (10, 20));
        assert!(cursor.attrs.bold);
    }

    #[test]
    fn test_cursor_visibility() {
        let mut cursor = Cursor::new();
        assert!(cursor.is_visible());

        cursor.hide();
        assert!(!cursor.is_visible());

        cursor.show();
        assert!(cursor.is_visible());

        cursor.toggle_visibility();
        assert!(!cursor.is_visible());
    }

    #[test]
    fn test_carriage_return() {
        let mut cursor = Cursor::new();
        cursor.move_to(50, 10);
        cursor.carriage_return();
        assert_eq!(cursor.position(), (0, 10));
    }

    #[test]
    fn test_new_line() {
        let mut cursor = Cursor::new();
        cursor.move_to(50, 10);
        cursor.new_line();
        assert_eq!(cursor.position(), (0, 11));
    }
}
