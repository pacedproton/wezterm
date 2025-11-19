//! Terminal modes and mode management
//!
//! Implements various terminal modes including DEC Private Mode (DECPM),
//! ANSI modes, and application-specific modes.

use std::collections::HashSet;

/// Terminal mode types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalMode {
    // ANSI Modes (SM/RM)
    /// Insert mode (IRM) - shift characters on insert
    Insert,
    /// Automatic newline (LNM) - CR causes CRLF
    AutomaticNewline,

    // DEC Private Modes (DECSET/DECRST)
    /// Application cursor keys (DECCKM)
    ApplicationCursorKeys,
    /// Application keypad (DECNKM)
    ApplicationKeypad,
    /// Reverse video (DECSCNM)
    ReverseVideo,
    /// Origin mode (DECOM) - relative to scrolling region
    OriginMode,
    /// Auto wrap (DECAWM)
    AutoWrap,
    /// Cursor visible (DECTCEM)
    CursorVisible,
    /// Mouse button press tracking
    MouseButtonPress,
    /// Mouse button press and release tracking
    MouseButtonPressRelease,
    /// Mouse motion tracking
    MouseMotion,
    /// Mouse motion and button tracking
    MouseMotionButton,
    /// SGR mouse mode
    SgrMouse,
    /// Alternate screen buffer (1049)
    AlternateScreen,
    /// Bracketed paste (2004)
    BracketedPaste,
    /// Focus reporting (1004)
    FocusReporting,
    /// Scrollbar mode
    Scrollbar,
    /// Cursor blink
    CursorBlink,

    // Custom modes
    /// Custom application mode
    Custom(u16),
}

impl TerminalMode {
    /// Get the mode number for CSI sequences
    pub fn mode_number(&self) -> Option<u16> {
        match self {
            // ANSI modes
            TerminalMode::Insert => Some(4),
            TerminalMode::AutomaticNewline => Some(20),

            // DEC Private modes (these use ? prefix)
            TerminalMode::ApplicationCursorKeys => Some(1),
            TerminalMode::ApplicationKeypad => Some(66),
            TerminalMode::ReverseVideo => Some(5),
            TerminalMode::OriginMode => Some(6),
            TerminalMode::AutoWrap => Some(7),
            TerminalMode::CursorVisible => Some(25),
            TerminalMode::MouseButtonPress => Some(1000),
            TerminalMode::MouseButtonPressRelease => Some(1002),
            TerminalMode::MouseMotion => Some(1003),
            TerminalMode::MouseMotionButton => Some(1006),
            TerminalMode::SgrMouse => Some(1006),
            TerminalMode::AlternateScreen => Some(1049),
            TerminalMode::BracketedPaste => Some(2004),
            TerminalMode::FocusReporting => Some(1004),
            TerminalMode::Scrollbar => Some(30),
            TerminalMode::CursorBlink => Some(12),

            TerminalMode::Custom(n) => Some(*n),
        }
    }

    /// Create mode from DEC private mode number
    pub fn from_dec_mode(number: u16) -> Option<Self> {
        match number {
            1 => Some(TerminalMode::ApplicationCursorKeys),
            5 => Some(TerminalMode::ReverseVideo),
            6 => Some(TerminalMode::OriginMode),
            7 => Some(TerminalMode::AutoWrap),
            12 => Some(TerminalMode::CursorBlink),
            25 => Some(TerminalMode::CursorVisible),
            30 => Some(TerminalMode::Scrollbar),
            66 => Some(TerminalMode::ApplicationKeypad),
            1000 => Some(TerminalMode::MouseButtonPress),
            1002 => Some(TerminalMode::MouseButtonPressRelease),
            1003 => Some(TerminalMode::MouseMotion),
            1004 => Some(TerminalMode::FocusReporting),
            1006 => Some(TerminalMode::SgrMouse),
            1049 => Some(TerminalMode::AlternateScreen),
            2004 => Some(TerminalMode::BracketedPaste),
            n => Some(TerminalMode::Custom(n)),
        }
    }

    /// Create mode from ANSI mode number
    pub fn from_ansi_mode(number: u16) -> Option<Self> {
        match number {
            4 => Some(TerminalMode::Insert),
            20 => Some(TerminalMode::AutomaticNewline),
            n => Some(TerminalMode::Custom(n)),
        }
    }

    /// Check if this is a DEC private mode
    pub fn is_dec_private(&self) -> bool {
        !matches!(self, TerminalMode::Insert | TerminalMode::AutomaticNewline)
    }
}

/// Mode manager for tracking terminal modes
#[derive(Debug, Default)]
pub struct ModeManager {
    /// Active modes
    active_modes: HashSet<TerminalMode>,
}

impl ModeManager {
    /// Create a new mode manager with default modes
    pub fn new() -> Self {
        let mut manager = Self {
            active_modes: HashSet::new(),
        };

        // Set default modes
        manager.set(TerminalMode::AutoWrap, true);
        manager.set(TerminalMode::CursorVisible, true);

        manager
    }

    /// Set a mode
    pub fn set(&mut self, mode: TerminalMode, enabled: bool) {
        if enabled {
            self.active_modes.insert(mode);
        } else {
            self.active_modes.remove(&mode);
        }
    }

    /// Check if a mode is active
    pub fn is_active(&self, mode: TerminalMode) -> bool {
        self.active_modes.contains(&mode)
    }

    /// Toggle a mode
    pub fn toggle(&mut self, mode: TerminalMode) {
        let is_active = self.is_active(mode);
        self.set(mode, !is_active);
    }

    /// Get all active modes
    pub fn active_modes(&self) -> impl Iterator<Item = &TerminalMode> {
        self.active_modes.iter()
    }

    /// Clear all modes
    pub fn clear(&mut self) {
        self.active_modes.clear();
    }

    /// Reset to default modes
    pub fn reset(&mut self) {
        self.clear();
        self.set(TerminalMode::AutoWrap, true);
        self.set(TerminalMode::CursorVisible, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_creation() {
        let mode = TerminalMode::ApplicationCursorKeys;
        assert_eq!(mode.mode_number(), Some(1));
        assert!(mode.is_dec_private());
    }

    #[test]
    fn test_from_dec_mode() {
        assert_eq!(
            TerminalMode::from_dec_mode(1),
            Some(TerminalMode::ApplicationCursorKeys)
        );
        assert_eq!(
            TerminalMode::from_dec_mode(25),
            Some(TerminalMode::CursorVisible)
        );
        assert_eq!(
            TerminalMode::from_dec_mode(1049),
            Some(TerminalMode::AlternateScreen)
        );
    }

    #[test]
    fn test_from_ansi_mode() {
        assert_eq!(TerminalMode::from_ansi_mode(4), Some(TerminalMode::Insert));
        assert_eq!(
            TerminalMode::from_ansi_mode(20),
            Some(TerminalMode::AutomaticNewline)
        );
    }

    #[test]
    fn test_mode_manager_set() {
        let mut manager = ModeManager::new();

        manager.set(TerminalMode::Insert, true);
        assert!(manager.is_active(TerminalMode::Insert));

        manager.set(TerminalMode::Insert, false);
        assert!(!manager.is_active(TerminalMode::Insert));
    }

    #[test]
    fn test_mode_manager_toggle() {
        let mut manager = ModeManager::new();

        let initial = manager.is_active(TerminalMode::Insert);
        manager.toggle(TerminalMode::Insert);
        assert_eq!(manager.is_active(TerminalMode::Insert), !initial);
    }

    #[test]
    fn test_mode_manager_default() {
        let manager = ModeManager::new();

        // Default modes should be active
        assert!(manager.is_active(TerminalMode::AutoWrap));
        assert!(manager.is_active(TerminalMode::CursorVisible));
    }

    #[test]
    fn test_mode_manager_reset() {
        let mut manager = ModeManager::new();

        manager.set(TerminalMode::Insert, true);
        manager.set(TerminalMode::ReverseVideo, true);

        manager.reset();

        assert!(!manager.is_active(TerminalMode::Insert));
        assert!(!manager.is_active(TerminalMode::ReverseVideo));
        assert!(manager.is_active(TerminalMode::AutoWrap));
        assert!(manager.is_active(TerminalMode::CursorVisible));
    }

    #[test]
    fn test_custom_mode() {
        let mode = TerminalMode::Custom(9999);
        assert_eq!(mode.mode_number(), Some(9999));

        let mut manager = ModeManager::new();
        manager.set(mode, true);
        assert!(manager.is_active(mode));
    }

    #[test]
    fn test_active_modes_iterator() {
        let mut manager = ModeManager::new();

        manager.set(TerminalMode::Insert, true);
        manager.set(TerminalMode::ReverseVideo, true);

        let active_count = manager.active_modes().count();
        assert!(active_count >= 2); // At least Insert and ReverseVideo
    }

    #[test]
    fn test_is_dec_private() {
        assert!(TerminalMode::ApplicationCursorKeys.is_dec_private());
        assert!(TerminalMode::AlternateScreen.is_dec_private());
        assert!(!TerminalMode::Insert.is_dec_private());
        assert!(!TerminalMode::AutomaticNewline.is_dec_private());
    }
}
