//! Terminal cell representation

use crate::color::ColorSpec;

/// A single terminal cell
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    /// Characters in this cell
    text: String,
    /// Cell attributes
    attrs: CellAttributes,
    /// Hyperlink (if any)
    hyperlink: Option<Hyperlink>,
}

impl Cell {
    /// Create a new cell with the given character
    pub fn new(ch: char) -> Self {
        Self {
            text: ch.to_string(),
            attrs: CellAttributes::default(),
            hyperlink: None,
        }
    }

    /// Create an empty/blank cell
    pub fn blank() -> Self {
        Self {
            text: " ".to_string(),
            attrs: CellAttributes::default(),
            hyperlink: None,
        }
    }

    /// Get the text content
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the text content
    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    /// Get cell attributes
    pub fn attrs(&self) -> &CellAttributes {
        &self.attrs
    }

    /// Get mutable cell attributes
    pub fn attrs_mut(&mut self) -> &mut CellAttributes {
        &mut self.attrs
    }

    /// Set cell attributes
    pub fn set_attrs(&mut self, attrs: CellAttributes) {
        self.attrs = attrs;
    }

    /// Get hyperlink if present
    pub fn hyperlink(&self) -> Option<&Hyperlink> {
        self.hyperlink.as_ref()
    }

    /// Set hyperlink
    pub fn set_hyperlink(&mut self, hyperlink: Option<Hyperlink>) {
        self.hyperlink = hyperlink;
    }

    /// Check if cell is blank
    pub fn is_blank(&self) -> bool {
        self.text.trim().is_empty() && self.hyperlink.is_none()
    }

    /// Get the width of this cell (1 for normal, 2 for wide)
    pub fn width(&self) -> u8 {
        unicode_width::UnicodeWidthStr::width(self.text.as_str()) as u8
    }

    /// Reset cell to blank state
    pub fn reset(&mut self) {
        self.text = " ".to_string();
        self.attrs = CellAttributes::default();
        self.hyperlink = None;
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::blank()
    }
}

/// Cell attributes (colors, styles)
#[derive(Debug, Clone, PartialEq)]
pub struct CellAttributes {
    /// Foreground color
    pub foreground: ColorSpec,
    /// Background color
    pub background: ColorSpec,
    /// Bold/Bright
    pub bold: bool,
    /// Italic
    pub italic: bool,
    /// Underline style
    pub underline: UnderlineStyle,
    /// Underline color (if different from foreground)
    pub underline_color: Option<ColorSpec>,
    /// Strikethrough
    pub strikethrough: bool,
    /// Blink
    pub blink: bool,
    /// Reverse video
    pub reverse: bool,
    /// Hidden/Invisible
    pub hidden: bool,
    /// Dim/Faint
    pub dim: bool,
}

impl Default for CellAttributes {
    fn default() -> Self {
        Self {
            foreground: ColorSpec::Default,
            background: ColorSpec::Default,
            bold: false,
            italic: false,
            underline: UnderlineStyle::None,
            underline_color: None,
            strikethrough: false,
            blink: false,
            reverse: false,
            hidden: false,
            dim: false,
        }
    }
}

impl CellAttributes {
    /// Create new default attributes
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset to default values
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Set foreground color
    pub fn set_foreground(&mut self, color: ColorSpec) -> &mut Self {
        self.foreground = color;
        self
    }

    /// Set background color
    pub fn set_background(&mut self, color: ColorSpec) -> &mut Self {
        self.background = color;
        self
    }

    /// Set bold
    pub fn set_bold(&mut self, bold: bool) -> &mut Self {
        self.bold = bold;
        self
    }

    /// Set italic
    pub fn set_italic(&mut self, italic: bool) -> &mut Self {
        self.italic = italic;
        self
    }

    /// Set underline style
    pub fn set_underline(&mut self, style: UnderlineStyle) -> &mut Self {
        self.underline = style;
        self
    }

    /// Set strikethrough
    pub fn set_strikethrough(&mut self, strikethrough: bool) -> &mut Self {
        self.strikethrough = strikethrough;
        self
    }

    /// Set reverse video
    pub fn set_reverse(&mut self, reverse: bool) -> &mut Self {
        self.reverse = reverse;
        self
    }

    /// Set dim/faint
    pub fn set_dim(&mut self, dim: bool) -> &mut Self {
        self.dim = dim;
        self
    }

    /// Set hidden/invisible
    pub fn set_hidden(&mut self, hidden: bool) -> &mut Self {
        self.hidden = hidden;
        self
    }

    /// Set blink
    pub fn set_blink(&mut self, blink: bool) -> &mut Self {
        self.blink = blink;
        self
    }
}

/// Underline styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnderlineStyle {
    /// No underline
    None,
    /// Single underline
    Single,
    /// Double underline
    Double,
    /// Curly/wavy underline
    Curly,
    /// Dotted underline
    Dotted,
    /// Dashed underline
    Dashed,
}

/// Hyperlink information (OSC 8)
#[derive(Debug, Clone, PartialEq)]
pub struct Hyperlink {
    /// URL of the hyperlink
    pub url: String,
    /// Optional ID for tracking
    pub id: Option<String>,
    /// Additional parameters
    pub params: Vec<(String, String)>,
}

impl Hyperlink {
    /// Create a new hyperlink with just a URL
    pub fn new(url: String) -> Self {
        Self {
            url,
            id: None,
            params: Vec::new(),
        }
    }

    /// Create a hyperlink with an ID
    pub fn with_id(url: String, id: String) -> Self {
        Self {
            url,
            id: Some(id),
            params: Vec::new(),
        }
    }

    /// Add a parameter
    pub fn add_param(&mut self, key: String, value: String) {
        self.params.push((key, value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_creation() {
        let cell = Cell::new('A');
        assert_eq!(cell.text(), "A");
        assert!(!cell.is_blank());
    }

    #[test]
    fn test_blank_cell() {
        let cell = Cell::blank();
        assert!(cell.is_blank());
        assert_eq!(cell.text(), " ");
    }

    #[test]
    fn test_cell_width() {
        let narrow = Cell::new('A');
        assert_eq!(narrow.width(), 1);

        let wide = Cell::new('漢');
        assert_eq!(wide.width(), 2);
    }

    #[test]
    fn test_cell_attributes() {
        let mut attrs = CellAttributes::new();
        attrs.set_bold(true).set_italic(true);
        assert!(attrs.bold);
        assert!(attrs.italic);
    }

    #[test]
    fn test_hyperlink() {
        let link = Hyperlink::new("https://example.com".to_string());
        assert_eq!(link.url, "https://example.com");
        assert!(link.id.is_none());
    }
}
