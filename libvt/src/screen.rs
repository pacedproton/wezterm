//! Screen buffer management

use crate::cell::Cell;
use std::fmt;

/// A single line of terminal cells
#[derive(Debug, Clone)]
pub struct Line {
    cells: Vec<Cell>,
    dirty: bool,
    wrapped: bool,
}

impl Line {
    /// Create a new line with the specified width
    pub fn new(width: usize) -> Self {
        Self {
            cells: vec![Cell::blank(); width],
            dirty: true,
            wrapped: false,
        }
    }

    /// Get the width of the line
    pub fn width(&self) -> usize {
        self.cells.len()
    }

    /// Get a cell at the specified column
    pub fn get_cell(&self, col: usize) -> Option<&Cell> {
        self.cells.get(col)
    }

    /// Get a mutable cell at the specified column
    pub fn get_cell_mut(&mut self, col: usize) -> Option<&mut Cell> {
        self.dirty = true;
        self.cells.get_mut(col)
    }

    /// Iterate over cells
    pub fn cells(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter()
    }

    /// Iterate over mutable cells
    pub fn cells_mut(&mut self) -> impl Iterator<Item = &mut Cell> {
        self.dirty = true;
        self.cells.iter_mut()
    }

    /// Check if line is dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear dirty flag
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Mark line as dirty
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if line wraps to next
    pub fn is_wrapped(&self) -> bool {
        self.wrapped
    }

    /// Set wrapped flag
    pub fn set_wrapped(&mut self, wrapped: bool) {
        self.wrapped = wrapped;
    }

    /// Resize line to new width
    pub fn resize(&mut self, new_width: usize) {
        if new_width > self.cells.len() {
            self.cells.resize(new_width, Cell::blank());
        } else {
            self.cells.truncate(new_width);
        }
        self.dirty = true;
    }

    /// Clear the line
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.reset();
        }
        self.dirty = true;
        self.wrapped = false;
    }

    /// Get text in range
    pub fn get_text_range(&self, start: usize, end: usize) -> String {
        self.cells[start.min(self.cells.len())..end.min(self.cells.len())]
            .iter()
            .map(super::cell::Cell::text)
            .collect()
    }

    /// Check if line has any hyperlinks
    pub fn has_hyperlinks(&self) -> bool {
        self.cells.iter().any(|c| c.hyperlink().is_some())
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for cell in &self.cells {
            write!(f, "{}", cell.text())?;
        }
        Ok(())
    }
}

/// Screen buffer
#[derive(Debug, Clone)]
pub struct Screen {
    lines: Vec<Line>,
    width: usize,
    height: usize,
    title: String,
    working_directory: Option<String>,
}

impl Screen {
    /// Create a new screen buffer
    pub fn new(width: usize, height: usize) -> Self {
        let lines = (0..height).map(|_| Line::new(width)).collect();

        Self {
            lines,
            width,
            height,
            title: String::new(),
            working_directory: None,
        }
    }

    /// Get screen dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Get width
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get height
    pub fn height(&self) -> usize {
        self.height
    }

    /// Resize screen
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        // Resize existing lines
        for line in &mut self.lines {
            line.resize(new_width);
        }

        // Add or remove lines
        if new_height > self.height {
            for _ in self.height..new_height {
                self.lines.push(Line::new(new_width));
            }
        } else {
            self.lines.truncate(new_height);
        }

        self.width = new_width;
        self.height = new_height;
    }

    /// Get a cell at the specified position
    pub fn get_cell(&self, col: usize, row: usize) -> Option<&Cell> {
        self.lines.get(row)?.get_cell(col)
    }

    /// Get a mutable cell at the specified position
    pub fn get_cell_mut(&mut self, col: usize, row: usize) -> Option<&mut Cell> {
        self.lines.get_mut(row)?.get_cell_mut(col)
    }

    /// Get a line at the specified row
    pub fn get_line(&self, row: usize) -> Option<&Line> {
        self.lines.get(row)
    }

    /// Get a mutable line at the specified row
    pub fn get_line_mut(&mut self, row: usize) -> Option<&mut Line> {
        self.lines.get_mut(row)
    }

    /// Iterate over all lines
    pub fn lines(&self) -> impl Iterator<Item = &Line> {
        self.lines.iter()
    }

    /// Iterate over mutable lines
    pub fn lines_mut(&mut self) -> impl Iterator<Item = &mut Line> {
        self.lines.iter_mut()
    }

    /// Remove a line and return it
    pub fn remove_line(&mut self, row: usize) -> Option<Line> {
        if row < self.lines.len() {
            Some(self.lines.remove(row))
        } else {
            None
        }
    }

    /// Push a new line at the bottom
    pub fn push_line(&mut self, line: Line) {
        self.lines.push(line);
    }

    /// Insert a line at the specified row
    pub fn insert_line(&mut self, row: usize, line: Line) {
        if row <= self.lines.len() {
            self.lines.insert(row, line);
        }
    }

    /// Clear the entire screen
    pub fn clear(&mut self) {
        for line in &mut self.lines {
            line.clear();
        }
    }

    /// Get terminal title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set terminal title
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Get working directory
    pub fn working_directory(&self) -> Option<&str> {
        self.working_directory.as_deref()
    }

    /// Set working directory
    pub fn set_working_directory(&mut self, dir: Option<String>) {
        self.working_directory = dir;
    }

    /// Get text in a range
    pub fn get_text_in_range(&self, start: Position, end: Position) -> String {
        let mut result = String::new();

        // Normalize so start < end
        let (start, end) = if start.row < end.row || (start.row == end.row && start.col <= end.col)
        {
            (start, end)
        } else {
            (end, start)
        };

        for row in start.row..=end.row {
            if let Some(line) = self.get_line(row as usize) {
                let start_col = if row == start.row {
                    start.col as usize
                } else {
                    0
                };
                let end_col = if row == end.row {
                    end.col as usize
                } else {
                    line.width()
                };

                result.push_str(&line.get_text_range(start_col, end_col));

                if row != end.row && !line.is_wrapped() {
                    result.push('\n');
                }
            }
        }

        result
    }

    /// Check if in alternate screen mode
    pub fn is_alternate(&self) -> bool {
        // This would need state tracking in a full implementation
        false
    }
}

/// Position in terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Column (0-indexed)
    pub col: u16,
    /// Row (0-indexed)
    pub row: u16,
}

impl Position {
    /// Create a new position
    pub fn new(col: u16, row: u16) -> Self {
        Self { col, row }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_creation() {
        let line = Line::new(80);
        assert_eq!(line.width(), 80);
        assert!(line.is_dirty());
    }

    #[test]
    fn test_line_cell_access() {
        let mut line = Line::new(10);
        let cell = line.get_cell_mut(5).unwrap();
        cell.set_text("X".to_string());

        assert_eq!(line.get_cell(5).unwrap().text(), "X");
    }

    #[test]
    fn test_line_to_string() {
        let line = Line::new(5);
        assert_eq!(line.to_string(), "     ");
    }

    #[test]
    fn test_screen_creation() {
        let screen = Screen::new(80, 24);
        assert_eq!(screen.dimensions(), (80, 24));
        assert_eq!(screen.width(), 80);
        assert_eq!(screen.height(), 24);
    }

    #[test]
    fn test_screen_cell_access() {
        let mut screen = Screen::new(80, 24);
        let cell = screen.get_cell_mut(10, 5).unwrap();
        cell.set_text("A".to_string());

        assert_eq!(screen.get_cell(10, 5).unwrap().text(), "A");
    }

    #[test]
    fn test_screen_resize() {
        let mut screen = Screen::new(80, 24);
        screen.resize(120, 40);
        assert_eq!(screen.dimensions(), (120, 40));
    }

    #[test]
    fn test_position() {
        let pos = Position::new(10, 20);
        assert_eq!(pos.col, 10);
        assert_eq!(pos.row, 20);
    }

    #[test]
    fn test_get_text_in_range() {
        let mut screen = Screen::new(10, 3);

        // Set up some text
        if let Some(cell) = screen.get_cell_mut(0, 0) {
            cell.set_text("H".to_string());
        }
        if let Some(cell) = screen.get_cell_mut(1, 0) {
            cell.set_text("i".to_string());
        }

        let text = screen.get_text_in_range(Position::new(0, 0), Position::new(2, 0));
        assert!(text.starts_with("Hi"));
    }
}
