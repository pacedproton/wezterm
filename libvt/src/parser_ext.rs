//! Parser extension API for custom escape sequence handlers
//!
//! Allows applications to register custom handlers for CSI, OSC, ESC, and DCS sequences.

use std::collections::HashMap;

/// Type of escape sequence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SequenceType {
    /// CSI (Control Sequence Introducer) - ESC [
    Csi,
    /// OSC (Operating System Command) - ESC ]
    Osc,
    /// ESC (Escape) - ESC followed by single character
    Esc,
    /// DCS (Device Control String) - ESC P
    Dcs,
}

/// Result of handling a custom sequence
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandlerResult {
    /// Sequence was handled successfully
    Handled,
    /// Sequence was not recognized by this handler
    NotHandled,
    /// Sequence caused an error
    Error(String),
}

/// Trait for custom escape sequence handlers
pub trait SequenceHandler: Send + Sync {
    /// Handle a sequence
    ///
    /// # Arguments
    /// * `sequence_type` - Type of sequence (CSI, OSC, etc.)
    /// * `params` - Sequence parameters/data
    ///
    /// # Returns
    /// `HandlerResult` indicating if the sequence was handled
    fn handle(&self, sequence_type: SequenceType, params: &[u8]) -> HandlerResult;

    /// Get the sequence identifier this handler responds to
    ///
    /// For CSI sequences, this might be 'm' for SGR (Set Graphics Rendition)
    /// For OSC sequences, this might be "1337" for iTerm2 proprietary sequences
    fn identifier(&self) -> &str;
}

/// Registry for custom escape sequence handlers
#[derive(Default)]
pub struct HandlerRegistry {
    /// CSI handlers by final byte
    csi_handlers: HashMap<String, Box<dyn SequenceHandler>>,
    /// OSC handlers by command number/name
    osc_handlers: HashMap<String, Box<dyn SequenceHandler>>,
    /// ESC handlers by character
    esc_handlers: HashMap<String, Box<dyn SequenceHandler>>,
    /// DCS handlers
    dcs_handlers: HashMap<String, Box<dyn SequenceHandler>>,
}

impl HandlerRegistry {
    /// Create a new handler registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a CSI handler
    ///
    /// # Example
    /// ```
    /// use libvt::parser_ext::{HandlerRegistry, SequenceHandler, SequenceType, HandlerResult};
    ///
    /// struct MyHandler;
    /// impl SequenceHandler for MyHandler {
    ///     fn handle(&self, _seq_type: SequenceType, _params: &[u8]) -> HandlerResult {
    ///         HandlerResult::Handled
    ///     }
    ///     fn identifier(&self) -> &str { "m" }
    /// }
    ///
    /// let mut registry = HandlerRegistry::new();
    /// registry.register_csi_handler(Box::new(MyHandler));
    /// ```
    pub fn register_csi_handler(&mut self, handler: Box<dyn SequenceHandler>) {
        let id = handler.identifier().to_string();
        self.csi_handlers.insert(id, handler);
    }

    /// Register an OSC handler
    pub fn register_osc_handler(&mut self, handler: Box<dyn SequenceHandler>) {
        let id = handler.identifier().to_string();
        self.osc_handlers.insert(id, handler);
    }

    /// Register an ESC handler
    pub fn register_esc_handler(&mut self, handler: Box<dyn SequenceHandler>) {
        let id = handler.identifier().to_string();
        self.esc_handlers.insert(id, handler);
    }

    /// Register a DCS handler
    pub fn register_dcs_handler(&mut self, handler: Box<dyn SequenceHandler>) {
        let id = handler.identifier().to_string();
        self.dcs_handlers.insert(id, handler);
    }

    /// Unregister a handler
    pub fn unregister_handler(&mut self, sequence_type: SequenceType, identifier: &str) -> bool {
        match sequence_type {
            SequenceType::Csi => self.csi_handlers.remove(identifier).is_some(),
            SequenceType::Osc => self.osc_handlers.remove(identifier).is_some(),
            SequenceType::Esc => self.esc_handlers.remove(identifier).is_some(),
            SequenceType::Dcs => self.dcs_handlers.remove(identifier).is_some(),
        }
    }

    /// Handle a sequence
    pub fn handle_sequence(
        &self,
        sequence_type: SequenceType,
        params: &[u8],
    ) -> Option<HandlerResult> {
        // Determine identifier from params
        let identifier = match sequence_type {
            SequenceType::Csi => {
                // For CSI, the final byte is the identifier
                params.last().map(|b| (*b as char).to_string())
            }
            SequenceType::Osc => {
                // For OSC, parse the command number
                let s = String::from_utf8_lossy(params);
                s.split(';').next().map(|s| s.to_string())
            }
            SequenceType::Esc => {
                // For ESC, the character after ESC is the identifier
                params.first().map(|b| (*b as char).to_string())
            }
            SequenceType::Dcs => {
                // For DCS, use the first character
                params.first().map(|b| (*b as char).to_string())
            }
        }?;

        let handler = match sequence_type {
            SequenceType::Csi => self.csi_handlers.get(&identifier)?,
            SequenceType::Osc => self.osc_handlers.get(&identifier)?,
            SequenceType::Esc => self.esc_handlers.get(&identifier)?,
            SequenceType::Dcs => self.dcs_handlers.get(&identifier)?,
        };

        Some(handler.handle(sequence_type, params))
    }

    /// Check if a handler is registered for a sequence type and identifier
    pub fn has_handler(&self, sequence_type: SequenceType, identifier: &str) -> bool {
        match sequence_type {
            SequenceType::Csi => self.csi_handlers.contains_key(identifier),
            SequenceType::Osc => self.osc_handlers.contains_key(identifier),
            SequenceType::Esc => self.esc_handlers.contains_key(identifier),
            SequenceType::Dcs => self.dcs_handlers.contains_key(identifier),
        }
    }

    /// Get the number of registered handlers
    pub fn handler_count(&self) -> usize {
        self.csi_handlers.len()
            + self.osc_handlers.len()
            + self.esc_handlers.len()
            + self.dcs_handlers.len()
    }

    /// Clear all handlers
    pub fn clear(&mut self) {
        self.csi_handlers.clear();
        self.osc_handlers.clear();
        self.esc_handlers.clear();
        self.dcs_handlers.clear();
    }
}

/// Helper for creating simple closure-based handlers
pub struct ClosureHandler<F>
where
    F: Fn(SequenceType, &[u8]) -> HandlerResult + Send + Sync,
{
    identifier: String,
    handler: F,
}

impl<F> ClosureHandler<F>
where
    F: Fn(SequenceType, &[u8]) -> HandlerResult + Send + Sync,
{
    /// Create a new closure-based handler
    pub fn new(identifier: impl Into<String>, handler: F) -> Self {
        Self {
            identifier: identifier.into(),
            handler,
        }
    }
}

impl<F> SequenceHandler for ClosureHandler<F>
where
    F: Fn(SequenceType, &[u8]) -> HandlerResult + Send + Sync,
{
    fn handle(&self, sequence_type: SequenceType, params: &[u8]) -> HandlerResult {
        (self.handler)(sequence_type, params)
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHandler {
        id: String,
        handled: bool,
    }

    impl TestHandler {
        fn new(id: impl Into<String>) -> Self {
            Self {
                id: id.into(),
                handled: true,
            }
        }
    }

    impl SequenceHandler for TestHandler {
        fn handle(&self, _sequence_type: SequenceType, _params: &[u8]) -> HandlerResult {
            if self.handled {
                HandlerResult::Handled
            } else {
                HandlerResult::NotHandled
            }
        }

        fn identifier(&self) -> &str {
            &self.id
        }
    }

    #[test]
    fn test_handler_result() {
        assert_eq!(HandlerResult::Handled, HandlerResult::Handled);
        assert_eq!(HandlerResult::NotHandled, HandlerResult::NotHandled);
        assert_ne!(HandlerResult::Handled, HandlerResult::NotHandled);
    }

    #[test]
    fn test_sequence_type() {
        assert_eq!(SequenceType::Csi, SequenceType::Csi);
        assert_ne!(SequenceType::Csi, SequenceType::Osc);
    }

    #[test]
    fn test_register_csi_handler() {
        let mut registry = HandlerRegistry::new();
        let handler = Box::new(TestHandler::new("m"));

        registry.register_csi_handler(handler);
        assert_eq!(registry.handler_count(), 1);
        assert!(registry.has_handler(SequenceType::Csi, "m"));
    }

    #[test]
    fn test_register_multiple_handlers() {
        let mut registry = HandlerRegistry::new();

        registry.register_csi_handler(Box::new(TestHandler::new("m")));
        registry.register_osc_handler(Box::new(TestHandler::new("1337")));
        registry.register_esc_handler(Box::new(TestHandler::new("7")));

        assert_eq!(registry.handler_count(), 3);
        assert!(registry.has_handler(SequenceType::Csi, "m"));
        assert!(registry.has_handler(SequenceType::Osc, "1337"));
        assert!(registry.has_handler(SequenceType::Esc, "7"));
    }

    #[test]
    fn test_unregister_handler() {
        let mut registry = HandlerRegistry::new();
        registry.register_csi_handler(Box::new(TestHandler::new("m")));

        assert_eq!(registry.handler_count(), 1);

        let removed = registry.unregister_handler(SequenceType::Csi, "m");
        assert!(removed);
        assert_eq!(registry.handler_count(), 0);

        let removed_again = registry.unregister_handler(SequenceType::Csi, "m");
        assert!(!removed_again);
    }

    #[test]
    fn test_handle_sequence_csi() {
        let mut registry = HandlerRegistry::new();
        registry.register_csi_handler(Box::new(TestHandler::new("m")));

        // CSI sequence ending in 'm'
        let params = b"31m";
        let result = registry.handle_sequence(SequenceType::Csi, params);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), HandlerResult::Handled);
    }

    #[test]
    fn test_handle_sequence_not_registered() {
        let registry = HandlerRegistry::new();

        let params = b"31m";
        let result = registry.handle_sequence(SequenceType::Csi, params);

        assert!(result.is_none());
    }

    #[test]
    fn test_clear_handlers() {
        let mut registry = HandlerRegistry::new();

        registry.register_csi_handler(Box::new(TestHandler::new("m")));
        registry.register_osc_handler(Box::new(TestHandler::new("1337")));

        assert_eq!(registry.handler_count(), 2);

        registry.clear();
        assert_eq!(registry.handler_count(), 0);
    }

    #[test]
    fn test_closure_handler() {
        let handler = ClosureHandler::new("test", |_seq_type, params| {
            if params.is_empty() {
                HandlerResult::NotHandled
            } else {
                HandlerResult::Handled
            }
        });

        assert_eq!(handler.identifier(), "test");
        assert_eq!(
            handler.handle(SequenceType::Csi, b""),
            HandlerResult::NotHandled
        );
        assert_eq!(
            handler.handle(SequenceType::Csi, b"abc"),
            HandlerResult::Handled
        );
    }

    #[test]
    fn test_closure_handler_in_registry() {
        let mut registry = HandlerRegistry::new();

        let handler = ClosureHandler::new("42", |_seq_type, _params| HandlerResult::Handled);

        registry.register_osc_handler(Box::new(handler));

        assert!(registry.has_handler(SequenceType::Osc, "42"));

        let result = registry.handle_sequence(SequenceType::Osc, b"42;test");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), HandlerResult::Handled);
    }

    #[test]
    fn test_error_result() {
        let handler = ClosureHandler::new("err", |_seq_type, _params| {
            HandlerResult::Error("Test error".to_string())
        });

        let result = handler.handle(SequenceType::Csi, b"test");
        assert!(matches!(result, HandlerResult::Error(_)));
    }
}
