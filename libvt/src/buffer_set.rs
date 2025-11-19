//! Buffer set for managing primary and alternate screen buffers
//!
//! The alternate screen buffer is used by full-screen applications like vim, less,
//! and tmux to preserve the main screen content while they run.

use crate::screen::Screen;

/// Identifies which buffer is active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferId {
    /// Primary (normal) screen buffer
    Primary,
    /// Alternate screen buffer (for full-screen apps)
    Alternate,
}

/// Manages primary and alternate screen buffers
///
/// Full-screen terminal applications (like vim, less, tmux) use the alternate
/// screen buffer to preserve the main terminal content. When they exit, the
/// original screen is restored.
///
/// # Example
///
/// ```rust
/// use libvt::buffer_set::{BufferSet, BufferId};
///
/// let mut buffers = BufferSet::new(80, 24);
///
/// // Initially on primary buffer
/// assert_eq!(buffers.active_buffer_id(), BufferId::Primary);
///
/// // Switch to alternate (like vim does)
/// buffers.switch_to_alternate();
/// assert_eq!(buffers.active_buffer_id(), BufferId::Alternate);
///
/// // Switch back to primary (vim exit)
/// buffers.switch_to_primary();
/// assert_eq!(buffers.active_buffer_id(), BufferId::Primary);
/// ```
#[derive(Debug)]
pub struct BufferSet {
    /// Primary screen buffer
    primary: Screen,
    /// Alternate screen buffer
    alternate: Screen,
    /// Currently active buffer
    active: BufferId,
}

impl BufferSet {
    /// Create a new buffer set with the specified dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            primary: Screen::new(width, height),
            alternate: Screen::new(width, height),
            active: BufferId::Primary,
        }
    }

    /// Get the currently active buffer ID
    pub fn active_buffer_id(&self) -> BufferId {
        self.active
    }

    /// Check if alternate screen is active
    pub fn is_alternate_active(&self) -> bool {
        self.active == BufferId::Alternate
    }

    /// Get a reference to the active buffer
    pub fn active(&self) -> &Screen {
        match self.active {
            BufferId::Primary => &self.primary,
            BufferId::Alternate => &self.alternate,
        }
    }

    /// Get a mutable reference to the active buffer
    pub fn active_mut(&mut self) -> &mut Screen {
        match self.active {
            BufferId::Primary => &mut self.primary,
            BufferId::Alternate => &mut self.alternate,
        }
    }

    /// Get a reference to the primary buffer
    pub fn primary(&self) -> &Screen {
        &self.primary
    }

    /// Get a mutable reference to the primary buffer
    pub fn primary_mut(&mut self) -> &mut Screen {
        &mut self.primary
    }

    /// Get a reference to the alternate buffer
    pub fn alternate(&self) -> &Screen {
        &self.alternate
    }

    /// Get a mutable reference to the alternate buffer
    pub fn alternate_mut(&mut self) -> &mut Screen {
        &mut self.alternate
    }

    /// Switch to the primary buffer
    ///
    /// Returns true if the buffer changed, false if already on primary
    pub fn switch_to_primary(&mut self) -> bool {
        if self.active == BufferId::Alternate {
            self.active = BufferId::Primary;
            true
        } else {
            false
        }
    }

    /// Switch to the alternate buffer
    ///
    /// Returns true if the buffer changed, false if already on alternate
    pub fn switch_to_alternate(&mut self) -> bool {
        if self.active == BufferId::Primary {
            self.active = BufferId::Alternate;
            // Clear alternate buffer when switching to it
            self.clear_alternate();
            true
        } else {
            false
        }
    }

    /// Clear the alternate buffer
    pub fn clear_alternate(&mut self) {
        for line in self.alternate.lines_mut() {
            line.clear();
        }
    }

    /// Resize both buffers
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        self.primary.resize(new_width, new_height);
        self.alternate.resize(new_width, new_height);
    }

    /// Get dimensions (width, height)
    pub fn dimensions(&self) -> (usize, usize) {
        self.primary.dimensions()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_set_creation() {
        let buffers = BufferSet::new(80, 24);
        assert_eq!(buffers.active_buffer_id(), BufferId::Primary);
        assert!(!buffers.is_alternate_active());
        assert_eq!(buffers.dimensions(), (80, 24));
    }

    #[test]
    fn test_switch_to_alternate() {
        let mut buffers = BufferSet::new(80, 24);

        // Initially on primary
        assert_eq!(buffers.active_buffer_id(), BufferId::Primary);

        // Switch to alternate
        assert!(buffers.switch_to_alternate());
        assert_eq!(buffers.active_buffer_id(), BufferId::Alternate);
        assert!(buffers.is_alternate_active());

        // Switching again should return false
        assert!(!buffers.switch_to_alternate());
    }

    #[test]
    fn test_switch_to_primary() {
        let mut buffers = BufferSet::new(80, 24);

        // Switch to alternate first
        buffers.switch_to_alternate();
        assert_eq!(buffers.active_buffer_id(), BufferId::Alternate);

        // Switch back to primary
        assert!(buffers.switch_to_primary());
        assert_eq!(buffers.active_buffer_id(), BufferId::Primary);
        assert!(!buffers.is_alternate_active());

        // Switching again should return false
        assert!(!buffers.switch_to_primary());
    }

    #[test]
    fn test_active_buffer_access() {
        let mut buffers = BufferSet::new(80, 24);

        // Access active buffer (primary)
        let active = buffers.active();
        assert_eq!(active.dimensions(), (80, 24));

        // Modify through active_mut
        {
            let active_mut = buffers.active_mut();
            if let Some(cell) = active_mut.get_cell_mut(0, 0) {
                cell.set_text("X".to_string());
            }
        }

        // Verify modification
        if let Some(cell) = buffers.active().get_cell(0, 0) {
            assert_eq!(cell.text(), "X");
        }
    }

    #[test]
    fn test_independent_buffers() {
        let mut buffers = BufferSet::new(80, 24);

        // Modify primary buffer
        if let Some(cell) = buffers.primary_mut().get_cell_mut(0, 0) {
            cell.set_text("P".to_string());
        }

        // Switch to alternate and modify
        buffers.switch_to_alternate();
        if let Some(cell) = buffers.alternate_mut().get_cell_mut(0, 0) {
            cell.set_text("A".to_string());
        }

        // Verify both buffers are independent
        if let Some(cell) = buffers.primary().get_cell(0, 0) {
            assert_eq!(cell.text(), "P");
        }
        if let Some(cell) = buffers.alternate().get_cell(0, 0) {
            assert_eq!(cell.text(), "A");
        }
    }

    #[test]
    fn test_resize() {
        let mut buffers = BufferSet::new(80, 24);

        buffers.resize(120, 40);

        assert_eq!(buffers.dimensions(), (120, 40));
        assert_eq!(buffers.primary().dimensions(), (120, 40));
        assert_eq!(buffers.alternate().dimensions(), (120, 40));
    }

    #[test]
    fn test_alternate_cleared_on_switch() {
        let mut buffers = BufferSet::new(80, 24);

        // Put some data in alternate buffer
        if let Some(cell) = buffers.alternate_mut().get_cell_mut(0, 0) {
            cell.set_text("X".to_string());
        }

        // Switch to alternate (should clear it)
        buffers.switch_to_alternate();

        // Verify it's cleared
        if let Some(cell) = buffers.alternate().get_cell(0, 0) {
            assert_eq!(cell.text(), " ");
        }
    }
}
