//! Character joiner system for custom grapheme clustering
//!
//! Allows applications to define custom rules for joining characters
//! into grapheme clusters, useful for ligatures, custom fonts, and
//! complex text rendering.

use std::sync::Arc;

/// Range of character positions to join
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JoinRange {
    /// Start column (inclusive)
    pub start: usize,
    /// End column (exclusive)
    pub end: usize,
}

impl JoinRange {
    /// Create a new join range
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Get the length of the range
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if the range is empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Check if this range overlaps with another
    pub fn overlaps(&self, other: &JoinRange) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Merge with another range if they overlap or are adjacent
    pub fn merge(&self, other: &JoinRange) -> Option<JoinRange> {
        if self.overlaps(other) || self.end == other.start || other.end == self.start {
            Some(JoinRange {
                start: self.start.min(other.start),
                end: self.end.max(other.end),
            })
        } else {
            None
        }
    }
}

/// Trait for character joiners
///
/// Implementations can define custom rules for joining characters
/// into grapheme clusters.
pub trait CharacterJoiner: Send + Sync {
    /// Check if characters should be joined
    ///
    /// # Arguments
    ///
    /// * `line_text` - The complete line text
    /// * `row` - The row number in the terminal
    ///
    /// # Returns
    ///
    /// Vector of ranges that should be joined
    fn join(&self, line_text: &str, row: usize) -> Vec<JoinRange>;

    /// Get the priority of this joiner (higher = applied first)
    fn priority(&self) -> i32 {
        0
    }
}

/// Ligature joiner - joins common programming ligatures
#[derive(Debug)]
pub struct LigatureJoiner {
    /// Ligature patterns to match
    patterns: Vec<&'static str>,
}

impl Default for LigatureJoiner {
    fn default() -> Self {
        Self::new()
    }
}

impl LigatureJoiner {
    /// Create a new ligature joiner with common programming ligatures
    pub fn new() -> Self {
        Self {
            patterns: vec![
                "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "::", "..", "...", "!!", "??",
                "++", "--", "/*", "*/", "//", "##", "**",
            ],
        }
    }

    /// Create with custom patterns
    pub fn with_patterns(patterns: Vec<&'static str>) -> Self {
        Self { patterns }
    }
}

impl CharacterJoiner for LigatureJoiner {
    fn join(&self, line_text: &str, _row: usize) -> Vec<JoinRange> {
        let mut ranges = Vec::new();

        for pattern in &self.patterns {
            let mut start = 0;
            while let Some(pos) = line_text[start..].find(pattern) {
                let absolute_pos = start + pos;
                ranges.push(JoinRange::new(
                    absolute_pos,
                    absolute_pos + pattern.len(),
                ));
                start = absolute_pos + 1;
            }
        }

        ranges
    }

    fn priority(&self) -> i32 {
        10
    }
}

/// Unicode combining marks joiner
#[derive(Debug, Default)]
pub struct CombiningMarksJoiner;

impl CombiningMarksJoiner {
    /// Create a new combining marks joiner
    pub fn new() -> Self {
        Self
    }

    /// Check if character is a combining mark
    fn is_combining_mark(ch: char) -> bool {
        // Unicode combining diacritical marks
        matches!(ch,
            '\u{0300}'..='\u{036F}' |  // Combining Diacritical Marks
            '\u{1AB0}'..='\u{1AFF}' |  // Combining Diacritical Marks Extended
            '\u{1DC0}'..='\u{1DFF}' |  // Combining Diacritical Marks Supplement
            '\u{20D0}'..='\u{20FF}' |  // Combining Diacritical Marks for Symbols
            '\u{FE20}'..='\u{FE2F}'    // Combining Half Marks
        )
    }
}

impl CharacterJoiner for CombiningMarksJoiner {
    fn join(&self, line_text: &str, _row: usize) -> Vec<JoinRange> {
        let mut ranges = Vec::new();
        let chars: Vec<(usize, char)> = line_text.char_indices().collect();

        let mut i = 0;
        while i < chars.len() {
            let (start, _) = chars[i];
            let mut end = start + chars[i].1.len_utf8();
            let mut joined = false;

            // Check for combining marks following this character
            let mut j = i + 1;
            while j < chars.len() && Self::is_combining_mark(chars[j].1) {
                end = chars[j].0 + chars[j].1.len_utf8();
                joined = true;
                j += 1;
            }

            if joined {
                ranges.push(JoinRange::new(start, end));
                i = j;
            } else {
                i += 1;
            }
        }

        ranges
    }

    fn priority(&self) -> i32 {
        100 // High priority - always apply combining marks
    }
}

/// Manager for multiple character joiners
#[derive(Default)]
pub struct CharacterJoinerManager {
    joiners: Vec<Arc<dyn CharacterJoiner>>,
}

impl CharacterJoinerManager {
    /// Create a new character joiner manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with default joiners (ligatures and combining marks)
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();
        manager.add_joiner(Arc::new(CombiningMarksJoiner::new()));
        manager.add_joiner(Arc::new(LigatureJoiner::new()));
        manager
    }

    /// Add a character joiner
    pub fn add_joiner(&mut self, joiner: Arc<dyn CharacterJoiner>) {
        self.joiners.push(joiner);
        // Sort by priority (highest first)
        self.joiners.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Get all join ranges for a line
    pub fn get_join_ranges(&self, line_text: &str, row: usize) -> Vec<JoinRange> {
        let mut all_ranges = Vec::new();

        for joiner in &self.joiners {
            all_ranges.extend(joiner.join(line_text, row));
        }

        // Merge overlapping ranges
        self.merge_ranges(all_ranges)
    }

    /// Merge overlapping or adjacent ranges
    fn merge_ranges(&self, mut ranges: Vec<JoinRange>) -> Vec<JoinRange> {
        if ranges.is_empty() {
            return ranges;
        }

        // Sort by start position
        ranges.sort_by_key(|r| r.start);

        let mut merged = vec![ranges[0]];

        for range in ranges.into_iter().skip(1) {
            let last = merged.last_mut().unwrap();

            if let Some(merged_range) = last.merge(&range) {
                *last = merged_range;
            } else {
                merged.push(range);
            }
        }

        merged
    }

    /// Clear all joiners
    pub fn clear(&mut self) {
        self.joiners.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_range_creation() {
        let range = JoinRange::new(5, 10);
        assert_eq!(range.start, 5);
        assert_eq!(range.end, 10);
        assert_eq!(range.len(), 5);
        assert!(!range.is_empty());
    }

    #[test]
    fn test_join_range_overlaps() {
        let r1 = JoinRange::new(5, 10);
        let r2 = JoinRange::new(8, 15);
        let r3 = JoinRange::new(20, 25);

        assert!(r1.overlaps(&r2));
        assert!(!r1.overlaps(&r3));
    }

    #[test]
    fn test_join_range_merge() {
        let r1 = JoinRange::new(5, 10);
        let r2 = JoinRange::new(8, 15);
        let r3 = JoinRange::new(20, 25);

        let merged = r1.merge(&r2);
        assert_eq!(merged, Some(JoinRange::new(5, 15)));

        let not_merged = r1.merge(&r3);
        assert_eq!(not_merged, None);

        // Adjacent ranges
        let r4 = JoinRange::new(10, 15);
        let adj_merged = r1.merge(&r4);
        assert_eq!(adj_merged, Some(JoinRange::new(5, 15)));
    }

    #[test]
    fn test_ligature_joiner() {
        let joiner = LigatureJoiner::new();
        let ranges = joiner.join("fn main() -> i32 { }", 0);

        // Should find "->"
        assert!(!ranges.is_empty());
        let arrow = ranges.iter().find(|r| r.len() == 2);
        assert!(arrow.is_some());
    }

    #[test]
    fn test_ligature_joiner_multiple() {
        let joiner = LigatureJoiner::new();
        let ranges = joiner.join("if a == b && c != d", 0);

        // Should find "==", "&&", "!="
        assert!(ranges.len() >= 3);
    }

    #[test]
    fn test_combining_marks_joiner() {
        let joiner = CombiningMarksJoiner::new();

        // "e" with combining acute accent (é)
        let text = "e\u{0301}";
        let ranges = joiner.join(text, 0);

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start, 0);
    }

    #[test]
    fn test_character_joiner_manager() {
        let mut manager = CharacterJoinerManager::with_defaults();
        let ranges = manager.get_join_ranges("fn test() -> bool", 0);

        // Should find "->"
        assert!(!ranges.is_empty());
    }

    #[test]
    fn test_merge_ranges() {
        let manager = CharacterJoinerManager::new();

        let ranges = vec![
            JoinRange::new(0, 5),
            JoinRange::new(3, 8),
            JoinRange::new(10, 15),
            JoinRange::new(14, 20),
        ];

        let merged = manager.merge_ranges(ranges);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], JoinRange::new(0, 8));
        assert_eq!(merged[1], JoinRange::new(10, 20));
    }

    #[test]
    fn test_custom_ligature_patterns() {
        let joiner = LigatureJoiner::with_patterns(vec!["<-", "<->", "=>>"]);
        let ranges = joiner.join("a <- b <-> c =>> d", 0);

        // Should find 4 matches: "<-" (2 times), "<->", and "=>>"
        // Note: "<-" appears both standalone and within "<->"
        assert_eq!(ranges.len(), 4);
    }

    #[test]
    fn test_is_combining_mark() {
        assert!(CombiningMarksJoiner::is_combining_mark('\u{0301}')); // Combining acute
        assert!(CombiningMarksJoiner::is_combining_mark('\u{0308}')); // Combining diaeresis
        assert!(!CombiningMarksJoiner::is_combining_mark('a'));
        assert!(!CombiningMarksJoiner::is_combining_mark('Z'));
    }

    #[test]
    fn test_empty_range() {
        let range = JoinRange::new(5, 5);
        assert!(range.is_empty());
        assert_eq!(range.len(), 0);
    }
}
