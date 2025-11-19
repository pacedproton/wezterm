//! Link detection and provider system
//!
//! Allows detecting and extracting URLs, file paths, and custom patterns
//! from terminal output for hyperlinking and interaction.

use regex::Regex;
use std::sync::Arc;

/// A detected link in terminal output
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    /// Start column (inclusive)
    pub start_col: usize,
    /// End column (exclusive)
    pub end_col: usize,
    /// Row number
    pub row: usize,
    /// Link text
    pub text: String,
    /// Link URI (may be different from text, e.g., file:// prefix)
    pub uri: String,
    /// Optional tooltip
    pub tooltip: Option<String>,
}

impl Link {
    /// Create a new link
    pub fn new(start_col: usize, end_col: usize, row: usize, text: String, uri: String) -> Self {
        Self {
            start_col,
            end_col,
            row,
            text,
            uri,
            tooltip: None,
        }
    }

    /// Create a link with a tooltip
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Check if a position is within this link
    pub fn contains(&self, col: usize, row: usize) -> bool {
        self.row == row && col >= self.start_col && col < self.end_col
    }
}

/// Trait for link providers
pub trait LinkProvider: Send + Sync {
    /// Detect links in a line of text
    ///
    /// # Arguments
    ///
    /// * `line` - The line text to scan
    /// * `row` - The row number in the terminal
    ///
    /// # Returns
    ///
    /// Vector of detected links
    fn find_links(&self, line: &str, row: usize) -> Vec<Link>;

    /// Get the priority of this provider (higher = checked first)
    fn priority(&self) -> i32 {
        0
    }
}

/// URL link provider - detects HTTP(S), FTP, etc.
#[derive(Debug)]
pub struct UrlLinkProvider {
    regex: Regex,
}

impl Default for UrlLinkProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl UrlLinkProvider {
    /// Create a new URL link provider
    pub fn new() -> Self {
        // Regex for detecting URLs
        let pattern = r"(?i)\b(?:https?|ftp)://[^\s<>{}|\\\^\[\]`]+";
        let regex = Regex::new(pattern).expect("Invalid URL regex");

        Self { regex }
    }
}

impl LinkProvider for UrlLinkProvider {
    fn find_links(&self, line: &str, row: usize) -> Vec<Link> {
        self.regex
            .find_iter(line)
            .map(|m| {
                Link::new(
                    m.start(),
                    m.end(),
                    row,
                    m.as_str().to_string(),
                    m.as_str().to_string(),
                )
            })
            .collect()
    }

    fn priority(&self) -> i32 {
        10
    }
}

/// File path link provider - detects file paths
#[derive(Debug)]
pub struct FilePathLinkProvider {
    regex: Regex,
}

impl Default for FilePathLinkProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FilePathLinkProvider {
    /// Create a new file path link provider
    pub fn new() -> Self {
        // Regex for detecting file paths (simplified)
        // Matches: /path/to/file or ./relative/path or ~/home/path
        let pattern = r"(?:[~/.])?(?:[a-zA-Z0-9_\-./]+/)+[a-zA-Z0-9_\-.]+(?:\.[a-zA-Z0-9]+)?";
        let regex = Regex::new(pattern).expect("Invalid file path regex");

        Self { regex }
    }
}

impl LinkProvider for FilePathLinkProvider {
    fn find_links(&self, line: &str, row: usize) -> Vec<Link> {
        self.regex
            .find_iter(line)
            .filter_map(|m| {
                let text = m.as_str();

                // Filter out very short matches (< 3 chars) to avoid false positives
                if text.len() < 3 {
                    return None;
                }

                // Convert to file:// URI
                let uri = if text.starts_with('/') {
                    format!("file://{text}")
                } else if text.starts_with("~/") {
                    // Expand ~ in URI (simple version)
                    format!("file://{}", text.replacen('~', "", 1))
                } else {
                    format!("file://./{text}")
                };

                Some(Link::new(m.start(), m.end(), row, text.to_string(), uri))
            })
            .collect()
    }

    fn priority(&self) -> i32 {
        5
    }
}

/// Custom regex-based link provider
#[derive(Debug)]
pub struct RegexLinkProvider {
    regex: Regex,
    uri_template: String,
    priority: i32,
}

impl RegexLinkProvider {
    /// Create a new regex link provider
    ///
    /// # Arguments
    ///
    /// * `pattern` - Regex pattern to match
    /// * `uri_template` - URI template (use $0 for full match, $1 for first group, etc.)
    ///
    /// # Example
    ///
    /// ```
    /// use libvt::link_provider::RegexLinkProvider;
    ///
    /// // Match GitHub issue references like #123
    /// let provider = RegexLinkProvider::new(
    ///     r"#(\d+)",
    ///     "https://github.com/owner/repo/issues/$1"
    /// ).unwrap();
    /// ```
    pub fn new(pattern: &str, uri_template: impl Into<String>) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern)?;
        Ok(Self {
            regex,
            uri_template: uri_template.into(),
            priority: 0,
        })
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

impl LinkProvider for RegexLinkProvider {
    fn find_links(&self, line: &str, row: usize) -> Vec<Link> {
        self.regex
            .find_iter(line)
            .map(|m| {
                let text = m.as_str().to_string();

                // Simple template substitution (just $0 for now)
                let uri = self.uri_template.replace("$0", &text);

                Link::new(m.start(), m.end(), row, text, uri)
            })
            .collect()
    }

    fn priority(&self) -> i32 {
        self.priority
    }
}

/// Manager for multiple link providers
#[derive(Default)]
pub struct LinkProviderManager {
    providers: Vec<Arc<dyn LinkProvider>>,
}

impl LinkProviderManager {
    /// Create a new link provider manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with default providers (URL and file path)
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();
        manager.add_provider(Arc::new(UrlLinkProvider::new()));
        manager.add_provider(Arc::new(FilePathLinkProvider::new()));
        manager
    }

    /// Add a link provider
    pub fn add_provider(&mut self, provider: Arc<dyn LinkProvider>) {
        self.providers.push(provider);
        // Sort by priority (highest first)
        self.providers
            .sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Find all links in a line
    pub fn find_links(&self, line: &str, row: usize) -> Vec<Link> {
        let mut all_links = Vec::new();

        for provider in &self.providers {
            all_links.extend(provider.find_links(line, row));
        }

        // Remove overlapping links (keep higher priority)
        self.deduplicate_links(all_links)
    }

    /// Remove overlapping links
    fn deduplicate_links(&self, mut links: Vec<Link>) -> Vec<Link> {
        if links.is_empty() {
            return links;
        }

        // Sort by start position
        links.sort_by_key(|l| l.start_col);

        let mut result = Vec::new();
        let mut last_end = 0;

        for link in links {
            // Skip if overlaps with previous link
            if link.start_col >= last_end {
                last_end = link.end_col;
                result.push(link);
            }
        }

        result
    }

    /// Clear all providers
    pub fn clear(&mut self) {
        self.providers.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_creation() {
        let link = Link::new(5, 10, 0, "https://example.com".to_string(), "https://example.com".to_string());
        assert_eq!(link.start_col, 5);
        assert_eq!(link.end_col, 10);
        assert_eq!(link.row, 0);
        assert!(link.contains(7, 0));
        assert!(!link.contains(11, 0));
        assert!(!link.contains(7, 1));
    }

    #[test]
    fn test_url_provider() {
        let provider = UrlLinkProvider::new();
        let links = provider.find_links("Visit https://example.com for more info", 0);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].text, "https://example.com");
        assert_eq!(links[0].uri, "https://example.com");
    }

    #[test]
    fn test_url_provider_multiple() {
        let provider = UrlLinkProvider::new();
        let links = provider.find_links(
            "See https://example.com and http://test.org",
            0,
        );

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].text, "https://example.com");
        assert_eq!(links[1].text, "http://test.org");
    }

    #[test]
    fn test_file_path_provider() {
        let provider = FilePathLinkProvider::new();
        let links = provider.find_links("Error in /usr/local/bin/app.rs:42", 0);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].text, "/usr/local/bin/app.rs");
        assert!(links[0].uri.starts_with("file://"));
    }

    #[test]
    fn test_regex_provider() {
        let provider = RegexLinkProvider::new(r"#(\d+)", "https://github.com/owner/repo/issues/$0")
            .unwrap();
        let links = provider.find_links("Fixed #123 and #456", 0);

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].text, "#123");
        assert!(links[0].uri.contains("#123"));
    }

    #[test]
    fn test_link_provider_manager() {
        let mut manager = LinkProviderManager::with_defaults();
        let links = manager.find_links("Visit https://example.com", 0);

        assert!(!links.is_empty());
        assert_eq!(links[0].text, "https://example.com");
    }

    #[test]
    fn test_deduplicate_links() {
        let manager = LinkProviderManager::new();

        let links = vec![
            Link::new(0, 5, 0, "test1".to_string(), "uri1".to_string()),
            Link::new(3, 8, 0, "test2".to_string(), "uri2".to_string()), // Overlaps with first
            Link::new(10, 15, 0, "test3".to_string(), "uri3".to_string()), // No overlap
        ];

        let deduped = manager.deduplicate_links(links);
        assert_eq!(deduped.len(), 2); // First and third should remain
        assert_eq!(deduped[0].text, "test1");
        assert_eq!(deduped[1].text, "test3");
    }

    #[test]
    fn test_link_with_tooltip() {
        let link = Link::new(0, 5, 0, "text".to_string(), "uri".to_string())
            .with_tooltip("Click to open");

        assert_eq!(link.tooltip, Some("Click to open".to_string()));
    }
}
