//! Terminal events and subscription system

use crate::{color::RgbColor, cursor::CursorShape, input::KeyModifiers as InputKeyModifiers};

/// Terminal events that can be subscribed to
#[derive(Debug, Clone)]
pub enum TerminalEvent {
    /// Data output from the terminal (xterm.js: onData)
    /// This is fired when the terminal has data to send (e.g., user input)
    Data(Vec<u8>),

    /// Binary data output from the terminal (xterm.js: onBinary)
    /// Similar to Data but explicitly for binary mode
    Binary(Vec<u8>),

    /// Key input event (xterm.js: onKey)
    /// Fired when a key is pressed
    Key {
        /// The key that was pressed
        key: String,
        /// The DOM event code (e.g., `KeyA`, `Enter`)
        code: String,
        /// Modifier state (shift, ctrl, alt, meta)
        modifiers: InputKeyModifiers,
    },

    /// Terminal was resized (xterm.js: onResize)
    Resized {
        /// New width in columns
        cols: u16,
        /// New height in rows
        rows: u16,
    },

    /// Scroll position changed (xterm.js: onScroll)
    /// Fired when the scroll position changes
    Scroll {
        /// New scroll position (0 = bottom, positive = scrolled up)
        position: u32,
    },

    /// Line feed occurred (xterm.js: onLineFeed)
    /// Fired whenever a line feed is added to the terminal
    LineFeed,

    /// Data was successfully parsed (xterm.js: onWriteParsed)
    /// Fired after data is written and parsed
    WriteParsed,

    /// Terminal bell rang (xterm.js: onBell)
    Bell,

    /// Title changed (via OSC 0/2)
    TitleChanged(String),

    /// Icon name changed (via OSC 1)
    IconNameChanged(String),

    /// Working directory changed (via OSC 7)
    WorkingDirectoryChanged(String),

    /// Color palette changed
    PaletteChanged,

    /// Cursor position changed
    CursorMoved {
        /// New column
        col: u16,
        /// New row
        row: u16,
    },

    /// Cursor shape changed
    CursorShapeChanged(CursorShape),

    /// Cursor visibility changed
    CursorVisibilityChanged(bool),

    /// Selection changed (xterm.js: onSelectionChange)
    SelectionChanged,

    /// Active buffer changed (primary/alternate)
    BufferChange {
        /// Whether alternate buffer is now active
        alternate: bool,
    },

    /// Clipboard request (OSC 52)
    ClipboardRequest {
        /// Clipboard type
        clipboard: ClipboardType,
        /// Data (base64 encoded)
        data: String,
    },

    /// Clipboard paste request
    ClipboardPaste {
        /// Clipboard type
        clipboard: ClipboardType,
    },

    /// Hyperlink activated
    HyperlinkActivated {
        /// URL of the hyperlink
        url: String,
    },

    /// Image loaded (Sixel/iTerm2/Kitty)
    ImageLoaded {
        /// Image identifier
        id: u32,
    },

    /// Output written to screen
    OutputWritten {
        /// Lines that were affected
        lines_affected: Vec<u16>,
    },

    /// Mode changed
    ModeChanged {
        /// Which mode changed
        mode: TerminalMode,
        /// Whether it was enabled or disabled
        enabled: bool,
    },

    /// Color query response (OSC 10/11)
    ColorResponse {
        /// Color index
        index: u8,
        /// Color value
        color: RgbColor,
    },

    /// Focus gained
    FocusGained,

    /// Focus lost
    FocusLost,

    /// Alternate screen enabled
    AlternateScreenEnabled,

    /// Alternate screen disabled
    AlternateScreenDisabled,

    /// Scrollback cleared
    ScrollbackCleared,
}

/// Event subscriber trait
pub trait EventSubscriber: Send + Sync {
    /// Called when an event occurs
    fn on_event(&self, event: &TerminalEvent);
}

/// Clipboard type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardType {
    /// System clipboard
    Clipboard,
    /// Primary selection (X11)
    Primary,
    /// Secondary selection
    Secondary,
    /// Select buffer
    Select,
    /// Cut buffer 0-7
    CutBuffer(u8),
}

/// Terminal modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalMode {
    /// Application cursor keys (DECCKM)
    CursorKeys,
    /// Insert/Replace mode (IRM)
    Insert,
    /// Send/Receive mode (SRM)
    SendReceive,
    /// Auto wrap mode (DECAWM)
    AutoWrap,
    /// Cursor blink (ATT610)
    CursorBlink,
    /// Bracketed paste mode
    BracketedPaste,
    /// Mouse tracking modes
    MouseTracking(MouseTrackingMode),
    /// Focus tracking
    FocusTracking,
    /// Alternate screen buffer
    AlternateScreen,
    /// Save cursor on alternate screen
    SaveCursorAlternateScreen,
    /// Origin mode (DECOM)
    OriginMode,
    /// Line feed/new line mode (LNM)
    LineFeedNewLine,
    /// Application keypad (DECKPAM)
    ApplicationKeypad,
    /// Reverse video (DECSCNM)
    ReverseVideo,
    /// Show cursor (DECTCEM)
    ShowCursor,
    /// Column 132 mode (DECCOLM)
    Column132,
}

/// Mouse tracking modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseTrackingMode {
    /// No mouse tracking
    None,
    /// X10 mouse reporting
    X10,
    /// Normal tracking (button press/release)
    Normal,
    /// Highlight tracking
    Highlight,
    /// Button event tracking (drag)
    ButtonEvent,
    /// Any event tracking
    AnyEvent,
    /// SGR encoding
    Sgr,
    /// UTF-8 encoding
    Utf8,
    /// URXVT encoding
    Urxvt,
}

/// Simple event callback wrapper
pub struct CallbackSubscriber<F>
where
    F: Fn(&TerminalEvent) + Send + Sync,
{
    callback: F,
}

impl<F> CallbackSubscriber<F>
where
    F: Fn(&TerminalEvent) + Send + Sync,
{
    /// Create a new callback subscriber
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

impl<F> EventSubscriber for CallbackSubscriber<F>
where
    F: Fn(&TerminalEvent) + Send + Sync,
{
    fn on_event(&self, event: &TerminalEvent) {
        (self.callback)(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_callback_subscriber() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let subscriber = CallbackSubscriber::new(move |event| {
            events_clone.lock().unwrap().push(event.clone());
        });

        subscriber.on_event(&TerminalEvent::Bell);
        subscriber.on_event(&TerminalEvent::TitleChanged("Test".to_string()));

        let recorded = events.lock().unwrap();
        assert_eq!(recorded.len(), 2);
        assert!(matches!(recorded[0], TerminalEvent::Bell));
        assert!(matches!(&recorded[1], TerminalEvent::TitleChanged(s) if s == "Test"));
    }

    #[test]
    fn test_terminal_mode() {
        let mode = TerminalMode::BracketedPaste;
        assert_eq!(mode, TerminalMode::BracketedPaste);
    }

    #[test]
    fn test_clipboard_type() {
        let clip = ClipboardType::Clipboard;
        assert_eq!(clip, ClipboardType::Clipboard);

        let cut = ClipboardType::CutBuffer(3);
        assert_eq!(cut, ClipboardType::CutBuffer(3));
    }

    #[test]
    fn test_data_event() {
        let data = vec![72, 101, 108, 108, 111]; // "Hello"
        let event = TerminalEvent::Data(data.clone());

        if let TerminalEvent::Data(d) = event {
            assert_eq!(d, data);
        } else {
            panic!("Expected Data event");
        }
    }

    #[test]
    fn test_binary_event() {
        let data = vec![0x00, 0x01, 0x02, 0xFF];
        let event = TerminalEvent::Binary(data.clone());

        if let TerminalEvent::Binary(d) = event {
            assert_eq!(d, data);
        } else {
            panic!("Expected Binary event");
        }
    }

    #[test]
    fn test_key_event() {
        let event = TerminalEvent::Key {
            key: "a".to_string(),
            code: "KeyA".to_string(),
            modifiers: InputKeyModifiers::CONTROL,
        };

        if let TerminalEvent::Key { key, code, modifiers } = event {
            assert_eq!(key, "a");
            assert_eq!(code, "KeyA");
            assert!(modifiers.contains(InputKeyModifiers::CONTROL));
            assert!(!modifiers.contains(InputKeyModifiers::SHIFT));
        } else {
            panic!("Expected Key event");
        }
    }

    #[test]
    fn test_scroll_event() {
        let event = TerminalEvent::Scroll { position: 42 };

        if let TerminalEvent::Scroll { position } = event {
            assert_eq!(position, 42);
        } else {
            panic!("Expected Scroll event");
        }
    }

    #[test]
    fn test_buffer_change_event() {
        let event = TerminalEvent::BufferChange { alternate: true };

        if let TerminalEvent::BufferChange { alternate } = event {
            assert!(alternate);
        } else {
            panic!("Expected BufferChange event");
        }
    }

    #[test]
    fn test_linefeed_event() {
        let event = TerminalEvent::LineFeed;
        assert!(matches!(event, TerminalEvent::LineFeed));
    }

    #[test]
    fn test_write_parsed_event() {
        let event = TerminalEvent::WriteParsed;
        assert!(matches!(event, TerminalEvent::WriteParsed));
    }
}
