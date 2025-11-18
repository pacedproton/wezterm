//! Error types for libvt

use std::fmt;

/// Result type for libvt operations
pub type Result<T> = std::result::Result<T, Error>;

/// `LibVT` error type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Invalid terminal dimensions
    InvalidDimensions {
        /// Number of columns
        cols: u16,
        /// Number of rows
        rows: u16,
    },

    /// Invalid position (out of bounds)
    InvalidPosition {
        /// Column position
        col: u16,
        /// Row position
        row: u16,
    },

    /// Invalid UTF-8 sequence
    InvalidUtf8,

    /// Invalid escape sequence
    InvalidEscapeSequence(String),

    /// Invalid color specification
    InvalidColor(String),

    /// Parser error
    ParserError(String),

    /// Buffer overflow
    BufferOverflow,

    /// Configuration error
    ConfigError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidDimensions { cols, rows } => {
                write!(f, "Invalid terminal dimensions: {cols}x{rows}")
            }
            Error::InvalidPosition { col, row } => {
                write!(f, "Invalid position: ({col}, {row})")
            }
            Error::InvalidUtf8 => write!(f, "Invalid UTF-8 sequence"),
            Error::InvalidEscapeSequence(seq) => {
                write!(f, "Invalid escape sequence: {seq}")
            }
            Error::InvalidColor(color) => write!(f, "Invalid color: {color}"),
            Error::ParserError(msg) => write!(f, "Parser error: {msg}"),
            Error::BufferOverflow => write!(f, "Buffer overflow"),
            Error::ConfigError(msg) => write!(f, "Configuration error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::InvalidDimensions { cols: 0, rows: 0 };
        assert_eq!(err.to_string(), "Invalid terminal dimensions: 0x0");

        let err = Error::InvalidPosition { col: 100, row: 200 };
        assert_eq!(err.to_string(), "Invalid position: (100, 200)");

        let err = Error::InvalidUtf8;
        assert_eq!(err.to_string(), "Invalid UTF-8 sequence");
    }

    #[test]
    fn test_error_equality() {
        let err1 = Error::InvalidUtf8;
        let err2 = Error::InvalidUtf8;
        assert_eq!(err1, err2);
    }
}
