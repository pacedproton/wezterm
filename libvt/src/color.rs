//! Color representation and palette management

/// Color specification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSpec {
    /// Default terminal color
    Default,
    /// ANSI color (0-15)
    Ansi(u8),
    /// 256-color palette index
    Palette(u8),
    /// True color RGB
    Rgb(RgbColor),
}

impl ColorSpec {
    /// Convert to CSS color string
    pub fn to_css(&self) -> String {
        match self {
            ColorSpec::Default => "inherit".to_string(),
            ColorSpec::Ansi(idx) => format!("var(--ansi-{})", idx),
            ColorSpec::Palette(idx) => format!("var(--palette-{})", idx),
            ColorSpec::Rgb(rgb) => rgb.to_css(),
        }
    }
}

/// RGB color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RgbColor {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
}

impl RgbColor {
    /// Create a new RGB color
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse from hex string (e.g., "#FF0000" or "FF0000")
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Self { r, g, b })
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Convert to CSS rgb() string
    pub fn to_css(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    /// Convert to CSS rgba() string with alpha
    pub fn to_css_rgba(&self, alpha: f32) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, alpha)
    }

    /// Blend with another color
    pub fn blend(&self, other: &RgbColor, factor: f32) -> RgbColor {
        let factor = factor.clamp(0.0, 1.0);
        let inv = 1.0 - factor;

        RgbColor {
            r: ((self.r as f32 * inv) + (other.r as f32 * factor)) as u8,
            g: ((self.g as f32 * inv) + (other.g as f32 * factor)) as u8,
            b: ((self.b as f32 * inv) + (other.b as f32 * factor)) as u8,
        }
    }

    /// Get luminance (0.0 - 1.0)
    pub fn luminance(&self) -> f32 {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Check if color is dark
    pub fn is_dark(&self) -> bool {
        self.luminance() < 0.5
    }

    /// Get contrasting color (black or white)
    pub fn contrasting(&self) -> RgbColor {
        if self.is_dark() {
            RgbColor::new(255, 255, 255)
        } else {
            RgbColor::new(0, 0, 0)
        }
    }
}

/// Color palette (256 colors)
#[derive(Debug, Clone)]
pub struct ColorPalette {
    /// 256-color palette
    pub colors: [RgbColor; 256],
}

impl Default for ColorPalette {
    fn default() -> Self {
        let mut colors = [RgbColor::new(0, 0, 0); 256];

        // Standard ANSI colors (0-7)
        colors[0] = RgbColor::new(0, 0, 0);         // Black
        colors[1] = RgbColor::new(204, 0, 0);       // Red
        colors[2] = RgbColor::new(78, 154, 6);      // Green
        colors[3] = RgbColor::new(196, 160, 0);     // Yellow
        colors[4] = RgbColor::new(52, 101, 164);    // Blue
        colors[5] = RgbColor::new(117, 80, 123);    // Magenta
        colors[6] = RgbColor::new(6, 152, 154);     // Cyan
        colors[7] = RgbColor::new(211, 215, 207);   // White

        // Bright ANSI colors (8-15)
        colors[8] = RgbColor::new(85, 87, 83);      // Bright Black
        colors[9] = RgbColor::new(239, 41, 41);     // Bright Red
        colors[10] = RgbColor::new(138, 226, 52);   // Bright Green
        colors[11] = RgbColor::new(252, 233, 79);   // Bright Yellow
        colors[12] = RgbColor::new(114, 159, 207);  // Bright Blue
        colors[13] = RgbColor::new(173, 127, 168);  // Bright Magenta
        colors[14] = RgbColor::new(52, 226, 226);   // Bright Cyan
        colors[15] = RgbColor::new(238, 238, 236);  // Bright White

        // 216 color cube (16-231)
        let mut idx = 16;
        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    let rv = if r == 0 { 0 } else { 55 + r * 40 };
                    let gv = if g == 0 { 0 } else { 55 + g * 40 };
                    let bv = if b == 0 { 0 } else { 55 + b * 40 };
                    colors[idx] = RgbColor::new(rv, gv, bv);
                    idx += 1;
                }
            }
        }

        // Grayscale (232-255)
        for i in 0..24 {
            let v = 8 + i * 10;
            colors[232 + i as usize] = RgbColor::new(v, v, v);
        }

        Self { colors }
    }
}

impl ColorPalette {
    /// Get color by index
    pub fn get(&self, index: u8) -> RgbColor {
        self.colors[index as usize]
    }

    /// Set color at index
    pub fn set(&mut self, index: u8, color: RgbColor) {
        self.colors[index as usize] = color;
    }

    /// Get ANSI color (0-15)
    pub fn ansi(&self, index: u8) -> RgbColor {
        self.colors[(index & 0x0F) as usize]
    }

    /// Set ANSI color (0-15)
    pub fn set_ansi(&mut self, index: u8, color: RgbColor) {
        self.colors[(index & 0x0F) as usize] = color;
    }

    /// Get foreground color (index 7 or 15 depending on brightness)
    pub fn foreground(&self) -> RgbColor {
        self.colors[7]
    }

    /// Get background color (index 0)
    pub fn background(&self) -> RgbColor {
        self.colors[0]
    }

    /// Set foreground color
    pub fn set_foreground(&mut self, color: RgbColor) {
        self.colors[7] = color;
        self.colors[15] = color; // Also set bright white
    }

    /// Set background color
    pub fn set_background(&mut self, color: RgbColor) {
        self.colors[0] = color;
    }

    /// Create a dark theme palette
    pub fn dark() -> Self {
        let mut palette = Self::default();
        palette.set_background(RgbColor::new(0, 0, 0));
        palette.set_foreground(RgbColor::new(229, 229, 229));
        palette
    }

    /// Create a light theme palette
    pub fn light() -> Self {
        let mut palette = Self::default();
        palette.set_background(RgbColor::new(255, 255, 255));
        palette.set_foreground(RgbColor::new(0, 0, 0));

        // Adjust ANSI colors for light background
        palette.colors[0] = RgbColor::new(0, 0, 0);
        palette.colors[7] = RgbColor::new(77, 77, 77);
        palette.colors[8] = RgbColor::new(102, 102, 102);
        palette.colors[15] = RgbColor::new(0, 0, 0);

        palette
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_from_hex() {
        let color = RgbColor::from_hex("#FF0000").unwrap();
        assert_eq!(color, RgbColor::new(255, 0, 0));

        let color = RgbColor::from_hex("00FF00").unwrap();
        assert_eq!(color, RgbColor::new(0, 255, 0));
    }

    #[test]
    fn test_rgb_to_hex() {
        let color = RgbColor::new(255, 128, 0);
        assert_eq!(color.to_hex(), "#ff8000");
    }

    #[test]
    fn test_rgb_luminance() {
        let black = RgbColor::new(0, 0, 0);
        assert!(black.is_dark());

        let white = RgbColor::new(255, 255, 255);
        assert!(!white.is_dark());
    }

    #[test]
    fn test_rgb_blend() {
        let black = RgbColor::new(0, 0, 0);
        let white = RgbColor::new(255, 255, 255);
        let gray = black.blend(&white, 0.5);
        assert_eq!(gray, RgbColor::new(127, 127, 127));
    }

    #[test]
    fn test_palette_default() {
        let palette = ColorPalette::default();
        // Black
        assert_eq!(palette.get(0), RgbColor::new(0, 0, 0));
        // White
        assert_eq!(palette.get(7), RgbColor::new(211, 215, 207));
    }

    #[test]
    fn test_color_spec_css() {
        let rgb = ColorSpec::Rgb(RgbColor::new(255, 0, 0));
        assert_eq!(rgb.to_css(), "rgb(255, 0, 0)");

        let ansi = ColorSpec::Ansi(1);
        assert_eq!(ansi.to_css(), "var(--ansi-1)");
    }
}
