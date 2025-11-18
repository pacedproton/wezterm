//! VT escape sequence parser

/// Parser state machine
pub struct Parser {
    state: ParserState,
    buffer: Vec<u8>,
    params: Vec<u8>,
}

/// Parser state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ParserState {
    /// Normal text processing
    #[default]
    Ground,
    /// Escape sequence started (received ESC)
    Escape,
    /// CSI sequence (ESC [)
    CsiEntry,
    /// CSI parameter bytes
    CsiParam,
    /// CSI intermediate bytes
    CsiIntermediate,
    /// OSC sequence (ESC ])
    OscString,
    /// DCS sequence (ESC P)
    DcsEntry,
    /// DCS passthrough
    DcsPassthrough,
    /// APC sequence (ESC _)
    ApcString,
    /// PM sequence (ESC ^)
    PmString,
    /// SOS sequence (ESC X)
    SosString,
}

/// Parser action result
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Print a character to screen
    Print(char),
    /// Control character (0x00-0x1F, 0x7F)
    Control(u8),
    /// CSI sequence
    Csi(Vec<u8>),
    /// Escape sequence
    Esc(Vec<u8>),
    /// OSC sequence
    Osc(Vec<u8>),
    /// DCS sequence
    Dcs(Vec<u8>),
    /// APC sequence
    Apc(Vec<u8>),
}

impl Parser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            state: ParserState::Ground,
            buffer: Vec::with_capacity(256),
            params: Vec::with_capacity(64),
        }
    }

    /// Parse input bytes and return actions
    pub fn parse(&mut self, input: &[u8]) -> Vec<Action> {
        let mut actions = Vec::new();

        for &byte in input {
            if let Some(action) = self.advance(byte) {
                actions.push(action);
            }
        }

        actions
    }

    /// Process a single byte
    fn advance(&mut self, byte: u8) -> Option<Action> {
        match self.state {
            ParserState::Ground => self.ground(byte),
            ParserState::Escape => self.escape(byte),
            ParserState::CsiEntry => self.csi_entry(byte),
            ParserState::CsiParam => self.csi_param(byte),
            ParserState::CsiIntermediate => self.csi_intermediate(byte),
            ParserState::OscString => self.osc_string(byte),
            ParserState::DcsEntry => self.dcs_entry(byte),
            ParserState::DcsPassthrough => self.dcs_passthrough(byte),
            ParserState::ApcString => self.apc_string(byte),
            ParserState::PmString => self.pm_string(byte),
            ParserState::SosString => self.sos_string(byte),
        }
    }

    fn ground(&mut self, byte: u8) -> Option<Action> {
        match byte {
            // ESC - must come before C0 control range
            0x1B => {
                self.state = ParserState::Escape;
                self.buffer.clear();
                None
            }
            // C0 control characters (excluding ESC which is handled above) and DEL
            0x00..=0x1A | 0x1C..=0x1F | 0x7F => Some(Action::Control(byte)),
            // Printable ASCII
            0x20..=0x7E => Some(Action::Print(byte as char)),
            // UTF-8 start bytes and continuation bytes
            0x80..=0xFF => {
                // Simple UTF-8 handling - just treat as printable for now
                // A full implementation would properly decode UTF-8 sequences
                char::from_u32(u32::from(byte)).map(Action::Print)
            }
        }
    }

    fn escape(&mut self, byte: u8) -> Option<Action> {
        match byte {
            // CSI (Control Sequence Introducer)
            b'[' => {
                self.state = ParserState::CsiEntry;
                self.buffer.clear();
                self.params.clear();
                None
            }
            // OSC (Operating System Command)
            b']' => {
                self.state = ParserState::OscString;
                self.buffer.clear();
                None
            }
            // DCS (Device Control String)
            b'P' => {
                self.state = ParserState::DcsEntry;
                self.buffer.clear();
                None
            }
            // APC (Application Program Command)
            b'_' => {
                self.state = ParserState::ApcString;
                self.buffer.clear();
                None
            }
            // PM (Privacy Message)
            b'^' => {
                self.state = ParserState::PmString;
                self.buffer.clear();
                None
            }
            // SOS (Start of String)
            b'X' => {
                self.state = ParserState::SosString;
                self.buffer.clear();
                None
            }
            // ST (String Terminator)
            b'\\' => {
                self.state = ParserState::Ground;
                None
            }
            // Simple escape sequences
            0x20..=0x7E => {
                self.state = ParserState::Ground;
                Some(Action::Esc(vec![byte]))
            }
            // ESC within ESC - restart
            0x1B => {
                self.buffer.clear();
                None
            }
            // Other - ignore and return to ground
            _ => {
                self.state = ParserState::Ground;
                None
            }
        }
    }

    fn csi_entry(&mut self, byte: u8) -> Option<Action> {
        match byte {
            // Parameter bytes
            0x30..=0x3F => {
                self.buffer.push(byte);
                self.state = ParserState::CsiParam;
                None
            }
            // Intermediate bytes
            0x20..=0x2F => {
                self.buffer.push(byte);
                self.state = ParserState::CsiIntermediate;
                None
            }
            // Final byte
            0x40..=0x7E => {
                self.buffer.push(byte);
                let action = Action::Csi(self.buffer.clone());
                self.state = ParserState::Ground;
                self.buffer.clear();
                Some(action)
            }
            // ESC - restart escape sequence
            0x1B => {
                self.state = ParserState::Escape;
                self.buffer.clear();
                None
            }
            // Other - ignore
            _ => None,
        }
    }

    fn csi_param(&mut self, byte: u8) -> Option<Action> {
        match byte {
            // More parameter bytes
            0x30..=0x3F => {
                self.buffer.push(byte);
                None
            }
            // Intermediate bytes
            0x20..=0x2F => {
                self.buffer.push(byte);
                self.state = ParserState::CsiIntermediate;
                None
            }
            // Final byte
            0x40..=0x7E => {
                self.buffer.push(byte);
                let action = Action::Csi(self.buffer.clone());
                self.state = ParserState::Ground;
                self.buffer.clear();
                Some(action)
            }
            // ESC - restart
            0x1B => {
                self.state = ParserState::Escape;
                self.buffer.clear();
                None
            }
            // Other - ignore
            _ => None,
        }
    }

    fn csi_intermediate(&mut self, byte: u8) -> Option<Action> {
        match byte {
            // More intermediate bytes
            0x20..=0x2F => {
                self.buffer.push(byte);
                None
            }
            // Final byte
            0x40..=0x7E => {
                self.buffer.push(byte);
                let action = Action::Csi(self.buffer.clone());
                self.state = ParserState::Ground;
                self.buffer.clear();
                Some(action)
            }
            // ESC - restart
            0x1B => {
                self.state = ParserState::Escape;
                self.buffer.clear();
                None
            }
            // Other - error, return to ground
            _ => {
                self.state = ParserState::Ground;
                self.buffer.clear();
                None
            }
        }
    }

    fn osc_string(&mut self, byte: u8) -> Option<Action> {
        match byte {
            // ST (String Terminator) - BEL
            0x07 => {
                let action = Action::Osc(self.buffer.clone());
                self.state = ParserState::Ground;
                self.buffer.clear();
                Some(action)
            }
            // ESC - might be ESC \\ (ST)
            0x1B => {
                // For simplicity, treat ESC as end of OSC
                // Real implementation would check for following backslash
                let action = Action::Osc(self.buffer.clone());
                self.state = ParserState::Escape;
                self.buffer.clear();
                Some(action)
            }
            // C0 controls (except BEL)
            0x00..=0x06 | 0x08..=0x1A | 0x1C..=0x1F => {
                // Ignore C0 controls in OSC
                None
            }
            // All other bytes are part of the string
            _ => {
                self.buffer.push(byte);
                None
            }
        }
    }

    fn dcs_entry(&mut self, byte: u8) -> Option<Action> {
        match byte {
            // Start passthrough
            0x40..=0x7E => {
                self.buffer.push(byte);
                self.state = ParserState::DcsPassthrough;
                None
            }
            // Parameter and intermediate bytes
            0x20..=0x3F => {
                self.buffer.push(byte);
                None
            }
            // ESC
            0x1B => {
                let action = Action::Dcs(self.buffer.clone());
                self.state = ParserState::Escape;
                self.buffer.clear();
                Some(action)
            }
            _ => None,
        }
    }

    fn dcs_passthrough(&mut self, byte: u8) -> Option<Action> {
        match byte {
            // ESC - end of DCS
            0x1B => {
                let action = Action::Dcs(self.buffer.clone());
                self.state = ParserState::Escape;
                self.buffer.clear();
                Some(action)
            }
            // ST via C1
            0x9C => {
                let action = Action::Dcs(self.buffer.clone());
                self.state = ParserState::Ground;
                self.buffer.clear();
                Some(action)
            }
            // All other bytes
            _ => {
                self.buffer.push(byte);
                None
            }
        }
    }

    fn apc_string(&mut self, byte: u8) -> Option<Action> {
        match byte {
            // ESC - end of APC
            0x1B => {
                let action = Action::Apc(self.buffer.clone());
                self.state = ParserState::Escape;
                self.buffer.clear();
                Some(action)
            }
            // ST via C1
            0x9C => {
                let action = Action::Apc(self.buffer.clone());
                self.state = ParserState::Ground;
                self.buffer.clear();
                Some(action)
            }
            // All other bytes
            _ => {
                self.buffer.push(byte);
                None
            }
        }
    }

    fn pm_string(&mut self, byte: u8) -> Option<Action> {
        match byte {
            // ESC - end of PM
            0x1B => {
                self.state = ParserState::Escape;
                self.buffer.clear();
                None
            }
            // ST via C1
            0x9C => {
                self.state = ParserState::Ground;
                self.buffer.clear();
                None
            }
            // Ignore all other bytes
            _ => {
                self.buffer.push(byte);
                None
            }
        }
    }

    fn sos_string(&mut self, byte: u8) -> Option<Action> {
        match byte {
            // ESC - end of SOS
            0x1B => {
                self.state = ParserState::Escape;
                self.buffer.clear();
                None
            }
            // ST via C1
            0x9C => {
                self.state = ParserState::Ground;
                self.buffer.clear();
                None
            }
            // Ignore all other bytes
            _ => {
                self.buffer.push(byte);
                None
            }
        }
    }

    /// Reset parser to initial state
    pub fn reset(&mut self) {
        self.state = ParserState::Ground;
        self.buffer.clear();
        self.params.clear();
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text() {
        let mut parser = Parser::new();
        let actions = parser.parse(b"Hello");

        assert_eq!(actions.len(), 5);
        assert_eq!(actions[0], Action::Print('H'));
        assert_eq!(actions[1], Action::Print('e'));
        assert_eq!(actions[2], Action::Print('l'));
        assert_eq!(actions[3], Action::Print('l'));
        assert_eq!(actions[4], Action::Print('o'));
    }

    #[test]
    fn test_parse_control_characters() {
        let mut parser = Parser::new();
        let actions = parser.parse(b"\x07\x08\x0D\x0A");

        assert_eq!(actions.len(), 4);
        assert_eq!(actions[0], Action::Control(0x07)); // Bell
        assert_eq!(actions[1], Action::Control(0x08)); // Backspace
        assert_eq!(actions[2], Action::Control(0x0D)); // CR
        assert_eq!(actions[3], Action::Control(0x0A)); // LF
    }

    #[test]
    fn test_parse_csi() {
        let mut parser = Parser::new();
        let actions = parser.parse(b"\x1b[31m");

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::Csi(b"31m".to_vec()));
    }

    #[test]
    fn test_parse_osc() {
        let mut parser = Parser::new();
        let actions = parser.parse(b"\x1b]0;Title\x07");

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::Osc(b"0;Title".to_vec()));
    }

    #[test]
    fn test_parse_simple_escape() {
        let mut parser = Parser::new();
        let actions = parser.parse(b"\x1bc");

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::Esc(vec![b'c']));
    }

    #[test]
    fn test_mixed_content() {
        let mut parser = Parser::new();
        let actions = parser.parse(b"Hi\x1b[31mRed\x07");

        assert_eq!(actions.len(), 7);
        assert_eq!(actions[0], Action::Print('H'));
        assert_eq!(actions[1], Action::Print('i'));
        assert_eq!(actions[2], Action::Csi(b"31m".to_vec()));
        assert_eq!(actions[3], Action::Print('R'));
        assert_eq!(actions[4], Action::Print('e'));
        assert_eq!(actions[5], Action::Print('d'));
        assert_eq!(actions[6], Action::Control(0x07));
    }
}
