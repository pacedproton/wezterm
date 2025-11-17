//! Input event handling

use bitflags::bitflags;

bitflags! {
    /// Keyboard modifiers
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct KeyModifiers: u8 {
        /// No modifiers
        const NONE = 0b0000_0000;
        /// Shift key
        const SHIFT = 0b0000_0001;
        /// Control key
        const CONTROL = 0b0000_0010;
        /// Alt/Option key
        const ALT = 0b0000_0100;
        /// Super/Cmd/Win key
        const SUPER = 0b0000_1000;
        /// Hyper key
        const HYPER = 0b0001_0000;
        /// Meta key
        const META = 0b0010_0000;
    }
}

// KeyModifiers::empty() is provided by bitflags macro
// Use KeyModifiers::empty() or KeyModifiers::NONE

/// Key codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// Character key
    Char(char),
    /// Function key (F1-F35)
    Function(u8),
    /// Backspace
    Backspace,
    /// Tab
    Tab,
    /// Enter/Return
    Enter,
    /// Escape
    Escape,
    /// Space
    Space,
    /// Page Up
    PageUp,
    /// Page Down
    PageDown,
    /// End
    End,
    /// Home
    Home,
    /// Left arrow
    Left,
    /// Up arrow
    Up,
    /// Right arrow
    Right,
    /// Down arrow
    Down,
    /// Insert
    Insert,
    /// Delete
    Delete,
    /// Numpad keys
    Numpad(u8),
    /// Numpad Enter
    NumpadEnter,
    /// Numpad +
    NumpadAdd,
    /// Numpad -
    NumpadSubtract,
    /// Numpad *
    NumpadMultiply,
    /// Numpad /
    NumpadDivide,
    /// Numpad .
    NumpadDecimal,
    /// Caps Lock
    CapsLock,
    /// Scroll Lock
    ScrollLock,
    /// Num Lock
    NumLock,
    /// Print Screen
    PrintScreen,
    /// Pause
    Pause,
    /// Menu/Application key
    Menu,
    /// Media Play/Pause
    MediaPlayPause,
    /// Media Stop
    MediaStop,
    /// Media Next Track
    MediaNextTrack,
    /// Media Previous Track
    MediaPrevTrack,
    /// Volume Up
    VolumeUp,
    /// Volume Down
    VolumeDown,
    /// Volume Mute
    VolumeMute,
}

/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// Left button
    Left,
    /// Middle button (wheel click)
    Middle,
    /// Right button
    Right,
    /// Scroll wheel up
    WheelUp,
    /// Scroll wheel down
    WheelDown,
    /// Scroll wheel left
    WheelLeft,
    /// Scroll wheel right
    WheelRight,
    /// Additional button (side button, etc.)
    Button(u8),
}

/// Mouse event kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseEventKind {
    /// Button pressed
    Press(MouseButton),
    /// Button released
    Release(MouseButton),
    /// Mouse moved (with button held)
    Drag(MouseButton),
    /// Mouse moved (no button)
    Move,
    /// Scroll wheel
    Scroll {
        /// Direction of scroll
        direction: MouseButton,
    },
}

/// Mouse event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseEvent {
    /// Column position (0-indexed, character cell)
    pub col: u16,
    /// Row position (0-indexed, character cell)
    pub row: u16,
    /// Event kind
    pub kind: MouseEventKind,
    /// Keyboard modifiers held during event
    pub modifiers: KeyModifiers,
}

impl MouseEvent {
    /// Create a new mouse event
    pub fn new(col: u16, row: u16, kind: MouseEventKind, modifiers: KeyModifiers) -> Self {
        Self {
            col,
            row,
            kind,
            modifiers,
        }
    }

    /// Create a press event
    pub fn press(col: u16, row: u16, button: MouseButton, modifiers: KeyModifiers) -> Self {
        Self {
            col,
            row,
            kind: MouseEventKind::Press(button),
            modifiers,
        }
    }

    /// Create a release event
    pub fn release(col: u16, row: u16, button: MouseButton, modifiers: KeyModifiers) -> Self {
        Self {
            col,
            row,
            kind: MouseEventKind::Release(button),
            modifiers,
        }
    }

    /// Create a drag event
    pub fn drag(col: u16, row: u16, button: MouseButton, modifiers: KeyModifiers) -> Self {
        Self {
            col,
            row,
            kind: MouseEventKind::Drag(button),
            modifiers,
        }
    }

    /// Create a move event
    pub fn move_event(col: u16, row: u16, modifiers: KeyModifiers) -> Self {
        Self {
            col,
            row,
            kind: MouseEventKind::Move,
            modifiers,
        }
    }

    /// Create a scroll event
    pub fn scroll(col: u16, row: u16, direction: MouseButton, modifiers: KeyModifiers) -> Self {
        Self {
            col,
            row,
            kind: MouseEventKind::Scroll { direction },
            modifiers,
        }
    }
}

/// Paste event (for bracketed paste)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteEvent {
    /// The pasted text
    pub text: String,
    /// Whether it's bracketed
    pub bracketed: bool,
}

impl PasteEvent {
    /// Create a new paste event
    pub fn new(text: String) -> Self {
        Self {
            text,
            bracketed: false,
        }
    }

    /// Create a bracketed paste event
    pub fn bracketed(text: String) -> Self {
        Self {
            text,
            bracketed: true,
        }
    }

    /// Encode for terminal input
    pub fn encode(&self) -> Vec<u8> {
        if self.bracketed {
            let mut result = b"\x1b[200~".to_vec();
            result.extend_from_slice(self.text.as_bytes());
            result.extend_from_slice(b"\x1b[201~");
            result
        } else {
            self.text.as_bytes().to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_modifiers() {
        let mods = KeyModifiers::SHIFT | KeyModifiers::CONTROL;
        assert!(mods.contains(KeyModifiers::SHIFT));
        assert!(mods.contains(KeyModifiers::CONTROL));
        assert!(!mods.contains(KeyModifiers::ALT));
    }

    #[test]
    fn test_mouse_event_creation() {
        let event = MouseEvent::press(10, 20, MouseButton::Left, KeyModifiers::empty());
        assert_eq!(event.col, 10);
        assert_eq!(event.row, 20);
        assert!(matches!(event.kind, MouseEventKind::Press(MouseButton::Left)));
    }

    #[test]
    fn test_paste_event_encoding() {
        let paste = PasteEvent::bracketed("Hello".to_string());
        let encoded = paste.encode();
        assert_eq!(encoded, b"\x1b[200~Hello\x1b[201~");
    }

    #[test]
    fn test_unbracketed_paste() {
        let paste = PasteEvent::new("Hello".to_string());
        let encoded = paste.encode();
        assert_eq!(encoded, b"Hello");
    }
}
