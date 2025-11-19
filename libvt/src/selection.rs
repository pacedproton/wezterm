//! Enhanced selection system with multiple modes
//!
//! Supports three selection modes matching xterm.js:
//! - Stream: Normal text selection (flows with line wrapping)
//! - Block: Rectangular/column selection
//! - Line: Selects entire lines

use crate::screen::{Position, Screen};

/// Selection mode determines how text is selected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Stream selection (normal text selection, follows line wrapping)
    Stream,
    /// Block/rectangular selection (selects a rectangular region)
    Block,
    /// Line selection (selects entire lines)
    Line,
}

/// Represents a text selection in the terminal
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// Selection mode
    mode: SelectionMode,
    /// Start position (anchor point)
    start: Position,
    /// End position (current cursor position)
    end: Position,
}

impl Selection {
    /// Create a new stream selection
    pub fn new_stream(start: Position, end: Position) -> Self {
        Self {
            mode: SelectionMode::Stream,
            start,
            end,
        }
    }

    /// Create a new block selection
    pub fn new_block(start: Position, end: Position) -> Self {
        Self {
            mode: SelectionMode::Block,
            start,
            end,
        }
    }

    /// Create a new line selection
    pub fn new_line(start: Position, end: Position) -> Self {
        Self {
            mode: SelectionMode::Line,
            start,
            end,
        }
    }

    /// Get selection mode
    pub fn mode(&self) -> SelectionMode {
        self.mode
    }

    /// Get start position
    pub fn start(&self) -> Position {
        self.start
    }

    /// Get end position
    pub fn end(&self) -> Position {
        self.end
    }

    /// Set selection mode
    pub fn set_mode(&mut self, mode: SelectionMode) {
        self.mode = mode;
    }

    /// Update selection end position
    pub fn update_end(&mut self, end: Position) {
        self.end = end;
    }

    /// Get normalized positions (start always before end)
    pub fn normalized(&self) -> (Position, Position) {
        let (mut start, mut end) = (self.start, self.end);

        // Normalize so start is always before end
        if start.row > end.row || (start.row == end.row && start.col > end.col) {
            std::mem::swap(&mut start, &mut end);
        }

        // For line mode, select entire lines
        if self.mode == SelectionMode::Line {
            start.col = 0;
            end.col = u16::MAX; // Will be clamped to line width
        }

        (start, end)
    }

    /// Check if a position is within the selection
    pub fn contains(&self, pos: Position) -> bool {
        let (start, end) = self.normalized();

        match self.mode {
            SelectionMode::Stream | SelectionMode::Line => {
                // Stream mode: position must be between start and end
                if pos.row < start.row || pos.row > end.row {
                    return false;
                }
                if pos.row == start.row && pos.col < start.col {
                    return false;
                }
                if pos.row == end.row && pos.col > end.col {
                    return false;
                }
                true
            }
            SelectionMode::Block => {
                // Block mode: position must be in rectangular region
                let min_col = start.col.min(end.col);
                let max_col = start.col.max(end.col);
                pos.row >= start.row && pos.row <= end.row && pos.col >= min_col && pos.col <= max_col
            }
        }
    }

    /// Extract selected text from screen
    pub fn get_text(&self, screen: &Screen) -> String {
        let (start, end) = self.normalized();

        match self.mode {
            SelectionMode::Stream => self.get_stream_text(screen, start, end),
            SelectionMode::Block => self.get_block_text(screen, start, end),
            SelectionMode::Line => self.get_line_text(screen, start, end),
        }
    }

    fn get_stream_text(&self, screen: &Screen, start: Position, end: Position) -> String {
        if start.row == end.row {
            // Single line selection
            if let Some(line) = screen.get_line(start.row as usize) {
                let end_col = (end.col as usize).min(line.width());
                return line.get_text_range(start.col as usize, end_col);
            }
            return String::new();
        }

        let mut result = String::new();

        // First line (from start.col to end)
        if let Some(line) = screen.get_line(start.row as usize) {
            result.push_str(&line.get_text_range(start.col as usize, line.width()));
            if !line.is_wrapped() {
                result.push('\n');
            }
        }

        // Middle lines (entire lines)
        for row in (start.row + 1)..end.row {
            if let Some(line) = screen.get_line(row as usize) {
                result.push_str(&line.get_text_range(0, line.width()));
                if !line.is_wrapped() {
                    result.push('\n');
                }
            }
        }

        // Last line (from start to end.col)
        if let Some(line) = screen.get_line(end.row as usize) {
            let end_col = (end.col as usize).min(line.width());
            result.push_str(&line.get_text_range(0, end_col));
        }

        result
    }

    fn get_block_text(&self, screen: &Screen, start: Position, end: Position) -> String {
        let mut result = String::new();
        let min_col = (start.col as usize).min(end.col as usize);
        let max_col = (start.col as usize).max(end.col as usize);

        for row in start.row..=end.row {
            if let Some(line) = screen.get_line(row as usize) {
                let start_col = min_col.min(line.width());
                // Add 1 to max_col because block selection is inclusive on both ends
                let end_col = (max_col + 1).min(line.width());
                if start_col < end_col {
                    result.push_str(&line.get_text_range(start_col, end_col));
                }
            }
            if row < end.row {
                result.push('\n');
            }
        }

        result
    }

    fn get_line_text(&self, screen: &Screen, start: Position, end: Position) -> String {
        let mut result = String::new();

        for row in start.row..=end.row {
            if let Some(line) = screen.get_line(row as usize) {
                result.push_str(&line.get_text_range(0, line.width()));
            }
            if row < end.row {
                result.push('\n');
            }
        }

        result
    }

    /// Trim trailing whitespace from selected text
    pub fn get_text_trimmed(&self, screen: &Screen) -> String {
        let text = self.get_text(screen);
        text.lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_screen() -> Screen {
        let mut screen = Screen::new(10, 5);

        // Fill with test data
        // Line 0: "Hello    "
        // Line 1: "World    "
        // Line 2: "Test123  "
        // Line 3: "ABCDEFGH "
        // Line 4: "12345    "

        let test_data = vec!["Hello", "World", "Test123", "ABCDEFGH", "12345"];

        for (row, text) in test_data.iter().enumerate() {
            for (col, ch) in text.chars().enumerate() {
                if let Some(cell) = screen.get_cell_mut(col, row) {
                    cell.set_text(ch.to_string());
                }
            }
        }

        screen
    }

    #[test]
    fn test_selection_modes() {
        let start = Position { row: 0, col: 0 };
        let end = Position { row: 1, col: 5 };

        let stream = Selection::new_stream(start, end);
        assert_eq!(stream.mode(), SelectionMode::Stream);

        let block = Selection::new_block(start, end);
        assert_eq!(block.mode(), SelectionMode::Block);

        let line = Selection::new_line(start, end);
        assert_eq!(line.mode(), SelectionMode::Line);
    }

    #[test]
    fn test_normalized() {
        // Forward selection
        let sel = Selection::new_stream(
            Position { row: 0, col: 5 },
            Position { row: 2, col: 3 },
        );
        let (start, end) = sel.normalized();
        assert_eq!(start, Position { row: 0, col: 5 });
        assert_eq!(end, Position { row: 2, col: 3 });

        // Backward selection (should swap)
        let sel = Selection::new_stream(
            Position { row: 2, col: 3 },
            Position { row: 0, col: 5 },
        );
        let (start, end) = sel.normalized();
        assert_eq!(start, Position { row: 0, col: 5 });
        assert_eq!(end, Position { row: 2, col: 3 });
    }

    #[test]
    fn test_line_mode_normalized() {
        let sel = Selection::new_line(
            Position { row: 1, col: 3 },
            Position { row: 2, col: 7 },
        );
        let (start, end) = sel.normalized();
        // Line mode should select from col 0 to max
        assert_eq!(start.col, 0);
        assert_eq!(end.col, u16::MAX);
    }

    #[test]
    fn test_contains_stream() {
        let sel = Selection::new_stream(
            Position { row: 1, col: 2 },
            Position { row: 3, col: 5 },
        );

        // Before selection
        assert!(!sel.contains(Position { row: 0, col: 5 }));
        assert!(!sel.contains(Position { row: 1, col: 1 }));

        // Inside selection
        assert!(sel.contains(Position { row: 1, col: 2 }));
        assert!(sel.contains(Position { row: 2, col: 0 }));
        assert!(sel.contains(Position { row: 3, col: 5 }));

        // After selection
        assert!(!sel.contains(Position { row: 3, col: 6 }));
        assert!(!sel.contains(Position { row: 4, col: 0 }));
    }

    #[test]
    fn test_contains_block() {
        let sel = Selection::new_block(
            Position { row: 1, col: 2 },
            Position { row: 3, col: 5 },
        );

        // Inside block
        assert!(sel.contains(Position { row: 1, col: 2 }));
        assert!(sel.contains(Position { row: 2, col: 3 }));
        assert!(sel.contains(Position { row: 3, col: 5 }));

        // Outside block (wrong column)
        assert!(!sel.contains(Position { row: 2, col: 1 }));
        assert!(!sel.contains(Position { row: 2, col: 6 }));

        // Outside block (wrong row)
        assert!(!sel.contains(Position { row: 0, col: 3 }));
        assert!(!sel.contains(Position { row: 4, col: 3 }));
    }

    #[test]
    fn test_stream_selection_single_line() {
        let screen = create_test_screen();
        let sel = Selection::new_stream(
            Position { row: 0, col: 1 },
            Position { row: 0, col: 4 },
        );

        let text = sel.get_text(&screen);
        assert_eq!(text, "ell");
    }

    #[test]
    fn test_stream_selection_multi_line() {
        let screen = create_test_screen();
        let sel = Selection::new_stream(
            Position { row: 0, col: 0 },
            Position { row: 1, col: 5 },
        );

        let text = sel.get_text(&screen);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn test_block_selection() {
        let screen = create_test_screen();
        // Select columns 1-3 across rows 0-2
        let sel = Selection::new_block(
            Position { row: 0, col: 1 },
            Position { row: 2, col: 3 },
        );

        let text = sel.get_text(&screen);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "ell"); // "Hello"[1..4]
        assert_eq!(lines[1], "orl"); // "World"[1..4]
        assert_eq!(lines[2], "est"); // "Test123"[1..4]
    }

    #[test]
    fn test_line_selection() {
        let screen = create_test_screen();
        // Select lines 1-2 (entire lines regardless of col)
        let sel = Selection::new_line(
            Position { row: 1, col: 3 },
            Position { row: 2, col: 1 },
        );

        let text = sel.get_text(&screen);
        assert!(text.contains("World"));
        assert!(text.contains("Test123"));
    }

    #[test]
    fn test_update_end() {
        let mut sel = Selection::new_stream(
            Position { row: 0, col: 0 },
            Position { row: 0, col: 5 },
        );

        sel.update_end(Position { row: 2, col: 3 });
        assert_eq!(sel.end(), Position { row: 2, col: 3 });
    }

    #[test]
    fn test_set_mode() {
        let mut sel = Selection::new_stream(
            Position { row: 0, col: 0 },
            Position { row: 1, col: 5 },
        );

        assert_eq!(sel.mode(), SelectionMode::Stream);

        sel.set_mode(SelectionMode::Block);
        assert_eq!(sel.mode(), SelectionMode::Block);
    }

    #[test]
    fn test_trimmed_text() {
        let screen = create_test_screen();
        let sel = Selection::new_stream(
            Position { row: 0, col: 0 },
            Position { row: 0, col: 9 },
        );

        let text = sel.get_text(&screen);
        let trimmed = sel.get_text_trimmed(&screen);

        // Original has trailing spaces
        assert!(text.ends_with(' '));
        // Trimmed does not
        assert_eq!(trimmed, "Hello");
    }
}
