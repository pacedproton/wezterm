//! Markers and decorations for terminal lines
//!
//! Markers allow applications to annotate specific lines in the terminal
//! for features like error indicators, breakpoints, search results, etc.

use std::collections::HashMap;

/// Unique identifier for a marker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarkerId(u64);

impl MarkerId {
    /// Create a new marker ID
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Type of decoration to apply to a marked line
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorationType {
    /// No specific decoration
    None,
    /// Error indicator (red)
    Error,
    /// Warning indicator (yellow)
    Warning,
    /// Info indicator (blue)
    Info,
    /// Success indicator (green)
    Success,
    /// Breakpoint indicator
    Breakpoint,
    /// Search result highlight
    SearchResult,
    /// Custom decoration with color
    Custom { r: u8, g: u8, b: u8 },
}

/// A marker attached to a specific line
#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    /// Unique identifier
    id: MarkerId,
    /// Line number (absolute row in scrollback + visible buffer)
    line: u32,
    /// Decoration type
    decoration: DecorationType,
    /// Optional message/tooltip
    message: Option<String>,
    /// Whether the marker is disposable (removed when line scrolls out)
    disposable: bool,
}

impl Marker {
    /// Create a new marker
    pub fn new(id: MarkerId, line: u32, decoration: DecorationType) -> Self {
        Self {
            id,
            line,
            decoration,
            message: None,
            disposable: true,
        }
    }

    /// Create a marker with a message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set whether the marker is disposable
    pub fn set_disposable(mut self, disposable: bool) -> Self {
        self.disposable = disposable;
        self
    }

    /// Get the marker ID
    pub fn id(&self) -> MarkerId {
        self.id
    }

    /// Get the line number
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Get the decoration type
    pub fn decoration(&self) -> DecorationType {
        self.decoration
    }

    /// Get the message if any
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Check if marker is disposable
    pub fn is_disposable(&self) -> bool {
        self.disposable
    }

    /// Update the line number (used when scrolling)
    pub(crate) fn update_line(&mut self, new_line: u32) {
        self.line = new_line;
    }
}

/// Manages markers and decorations for the terminal
#[derive(Debug)]
pub struct MarkerManager {
    /// All markers by ID
    markers: HashMap<MarkerId, Marker>,
    /// Next marker ID to assign
    next_id: u64,
}

impl Default for MarkerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkerManager {
    /// Create a new marker manager
    pub fn new() -> Self {
        Self {
            markers: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add a marker at the specified line
    pub fn add_marker(&mut self, line: u32, decoration: DecorationType) -> MarkerId {
        let id = MarkerId::new(self.next_id);
        self.next_id += 1;

        let marker = Marker::new(id, line, decoration);
        self.markers.insert(id, marker);
        id
    }

    /// Add a marker with a message
    pub fn add_marker_with_message(
        &mut self,
        line: u32,
        decoration: DecorationType,
        message: impl Into<String>,
    ) -> MarkerId {
        let id = MarkerId::new(self.next_id);
        self.next_id += 1;

        let marker = Marker::new(id, line, decoration).with_message(message);
        self.markers.insert(id, marker);
        id
    }

    /// Remove a marker by ID
    pub fn remove_marker(&mut self, id: MarkerId) -> Option<Marker> {
        self.markers.remove(&id)
    }

    /// Get a marker by ID
    pub fn get_marker(&self, id: MarkerId) -> Option<&Marker> {
        self.markers.get(&id)
    }

    /// Get all markers for a specific line
    pub fn get_markers_for_line(&self, line: u32) -> Vec<&Marker> {
        self.markers
            .values()
            .filter(|m| m.line == line)
            .collect()
    }

    /// Get all markers
    pub fn markers(&self) -> impl Iterator<Item = &Marker> {
        self.markers.values()
    }

    /// Clear all markers
    pub fn clear(&mut self) {
        self.markers.clear();
    }

    /// Update marker positions when scrolling
    ///
    /// When lines scroll up, markers need to move up with them or be removed
    /// if they scroll out of the buffer.
    pub fn handle_scroll(&mut self, lines_scrolled: u32) {
        let mut to_remove = Vec::new();

        for (id, marker) in &mut self.markers {
            if marker.disposable {
                if marker.line >= lines_scrolled {
                    marker.line -= lines_scrolled;
                } else {
                    // Marker scrolled out, mark for removal
                    to_remove.push(*id);
                }
            }
        }

        // Remove markers that scrolled out
        for id in to_remove {
            self.markers.remove(&id);
        }
    }

    /// Get the total number of markers
    pub fn len(&self) -> usize {
        self.markers.len()
    }

    /// Check if there are no markers
    pub fn is_empty(&self) -> bool {
        self.markers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_creation() {
        let marker = Marker::new(MarkerId::new(1), 10, DecorationType::Error);
        assert_eq!(marker.id(), MarkerId::new(1));
        assert_eq!(marker.line(), 10);
        assert_eq!(marker.decoration(), DecorationType::Error);
        assert!(marker.message().is_none());
        assert!(marker.is_disposable());
    }

    #[test]
    fn test_marker_with_message() {
        let marker = Marker::new(MarkerId::new(1), 10, DecorationType::Warning)
            .with_message("Test warning");

        assert_eq!(marker.message(), Some("Test warning"));
    }

    #[test]
    fn test_marker_disposable() {
        let marker = Marker::new(MarkerId::new(1), 10, DecorationType::Info)
            .set_disposable(false);

        assert!(!marker.is_disposable());
    }

    #[test]
    fn test_marker_manager_add() {
        let mut manager = MarkerManager::new();

        let id1 = manager.add_marker(5, DecorationType::Error);
        let id2 = manager.add_marker(10, DecorationType::Warning);

        assert_eq!(manager.len(), 2);
        assert!(manager.get_marker(id1).is_some());
        assert!(manager.get_marker(id2).is_some());
    }

    #[test]
    fn test_marker_manager_remove() {
        let mut manager = MarkerManager::new();

        let id = manager.add_marker(5, DecorationType::Error);
        assert_eq!(manager.len(), 1);

        let removed = manager.remove_marker(id);
        assert!(removed.is_some());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_get_markers_for_line() {
        let mut manager = MarkerManager::new();

        manager.add_marker(5, DecorationType::Error);
        manager.add_marker(5, DecorationType::Warning);
        manager.add_marker(10, DecorationType::Info);

        let markers = manager.get_markers_for_line(5);
        assert_eq!(markers.len(), 2);

        let markers = manager.get_markers_for_line(10);
        assert_eq!(markers.len(), 1);

        let markers = manager.get_markers_for_line(99);
        assert_eq!(markers.len(), 0);
    }

    #[test]
    fn test_handle_scroll() {
        let mut manager = MarkerManager::new();

        manager.add_marker(5, DecorationType::Error);
        manager.add_marker(10, DecorationType::Warning);
        manager.add_marker(15, DecorationType::Info);

        // Scroll by 3 lines
        manager.handle_scroll(3);

        // All markers should move up by 3
        let markers: Vec<_> = manager.markers().collect();
        assert_eq!(markers.len(), 3);

        // Check that lines were adjusted
        let lines: Vec<u32> = markers.iter().map(|m| m.line()).collect();
        assert!(lines.contains(&2)); // 5 - 3 = 2
        assert!(lines.contains(&7)); // 10 - 3 = 7
        assert!(lines.contains(&12)); // 15 - 3 = 12
    }

    #[test]
    fn test_handle_scroll_removes_scrolled_out() {
        let mut manager = MarkerManager::new();

        manager.add_marker(2, DecorationType::Error);
        manager.add_marker(10, DecorationType::Warning);

        // Scroll by 5 lines - first marker should be removed
        manager.handle_scroll(5);

        assert_eq!(manager.len(), 1);

        // Only the warning marker should remain
        let remaining = manager.markers().next().unwrap();
        assert_eq!(remaining.decoration(), DecorationType::Warning);
        assert_eq!(remaining.line(), 5); // 10 - 5 = 5
    }

    #[test]
    fn test_non_disposable_markers_persist() {
        let mut manager = MarkerManager::new();

        let id = manager.add_marker(2, DecorationType::Breakpoint);
        if let Some(marker) = manager.markers.get_mut(&id) {
            marker.disposable = false;
        }

        manager.add_marker(10, DecorationType::Warning);

        // Scroll past the breakpoint
        manager.handle_scroll(5);

        // Non-disposable marker should still exist (but not moved)
        assert_eq!(manager.len(), 2);

        let breakpoint = manager.get_marker(id).unwrap();
        assert_eq!(breakpoint.line(), 2); // Line number unchanged for non-disposable
    }

    #[test]
    fn test_clear_markers() {
        let mut manager = MarkerManager::new();

        manager.add_marker(5, DecorationType::Error);
        manager.add_marker(10, DecorationType::Warning);

        assert_eq!(manager.len(), 2);

        manager.clear();
        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_decoration_types() {
        assert_eq!(DecorationType::Error, DecorationType::Error);
        assert_ne!(DecorationType::Error, DecorationType::Warning);

        let custom1 = DecorationType::Custom {
            r: 255,
            g: 0,
            b: 0,
        };
        let custom2 = DecorationType::Custom {
            r: 255,
            g: 0,
            b: 0,
        };
        assert_eq!(custom1, custom2);
    }

    #[test]
    fn test_marker_id() {
        let id1 = MarkerId::new(1);
        let id2 = MarkerId::new(2);

        assert_eq!(id1.as_u64(), 1);
        assert_eq!(id2.as_u64(), 2);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_marker_update_line() {
        let mut marker = Marker::new(MarkerId::new(1), 10, DecorationType::Error);
        assert_eq!(marker.line(), 10);

        marker.update_line(20);
        assert_eq!(marker.line(), 20);
    }
}
