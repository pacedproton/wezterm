# LibVT MVP Implementation Plan - VSCode Integration
## Full 5-Phase Development Roadmap

**Version**: 1.0
**Target**: VSCode Terminal Integration via NAPI-RS
**Timeline**: 25-34 weeks (6-8 months)
**Current Status**: Foundation complete (3,162 LOC, 52 tests)

---

## Executive Summary

This document provides a complete implementation plan to transform libvt from a foundational terminal emulator (~15% feature complete vs xterm.js) into a production-ready VSCode terminal backend.

### Goals

1. **xterm.js API Compatibility**: Match 55+ methods, 12 events, core functionality
2. **Performance**: 2-4x faster parsing, 2x better memory efficiency
3. **Safety**: 100% safe Rust, no undefined behavior
4. **Integration**: Seamless NAPI-RS bridge to TypeScript
5. **Quality**: Comprehensive testing, documentation, benchmarks

### Success Criteria

- ✅ All critical xterm.js APIs implemented
- ✅ vim/emacs/tmux work correctly
- ✅ VSCode terminal functional with libvt backend
- ✅ Performance: >100 MB/s parse throughput, <100 MB memory for 100k scrollback
- ✅ Tests: >200 unit tests, >50 integration tests, vttest compatible
- ✅ Zero unsafe code, zero clippy warnings

---

## Project Structure

### Repository Layout

```
wezterm/
├── libvt/                          # Core Rust library
│   ├── src/
│   │   ├── lib.rs                  # Public API exports
│   │   ├── error.rs                # ✅ Error types (done)
│   │   ├── terminal.rs             # ✅ Terminal core (partial)
│   │   ├── screen.rs               # ✅ Screen buffer (partial)
│   │   ├── cell.rs                 # ✅ Cell types (done)
│   │   ├── color.rs                # ✅ Color support (done)
│   │   ├── cursor.rs               # ✅ Cursor state (done)
│   │   ├── input.rs                # ✅ Input handling (partial)
│   │   ├── parser/                 # ✅ Parser (done)
│   │   ├── events.rs               # ✅ Event system (partial)
│   │   │
│   │   ├── buffer.rs               # ❌ NEW: Buffer abstraction
│   │   ├── alt_screen.rs           # ❌ NEW: Alternate screen
│   │   ├── selection.rs            # ❌ NEW: Enhanced selection
│   │   ├── scrollback.rs           # ❌ NEW: Scrollback management
│   │   ├── markers.rs              # ❌ NEW: Marker system
│   │   ├── modes.rs                # ❌ NEW: Terminal modes
│   │   ├── unicode.rs              # ❌ NEW: Unicode handling
│   │   ├── links.rs                # ❌ NEW: Link detection
│   │   └── charset.rs              # ❌ NEW: Character sets
│   │
│   ├── benches/                    # ❌ NEW: Benchmarks
│   ├── tests/                      # ⚠️  Integration tests
│   ├── examples/                   # ✅ Demo (partial)
│   └── Cargo.toml
│
├── libvt-napi/                     # ❌ NEW: NAPI-RS bindings
│   ├── src/
│   │   ├── lib.rs                  # NAPI exports
│   │   ├── terminal.rs             # Terminal wrapper
│   │   ├── buffer.rs               # Buffer API
│   │   ├── events.rs               # Event callbacks
│   │   └── options.rs              # Configuration
│   ├── index.d.ts                  # TypeScript definitions
│   ├── package.json
│   └── Cargo.toml
│
├── libvt-renderer/                 # ❌ NEW: Canvas/WebGL renderer
│   ├── src/
│   │   ├── renderer.ts             # Main renderer
│   │   ├── layers/
│   │   │   ├── text.ts             # Text layer
│   │   │   ├── selection.ts        # Selection layer
│   │   │   ├── cursor.ts           # Cursor layer
│   │   │   └── links.ts            # Link layer
│   │   ├── atlas.ts                # Glyph atlas
│   │   └── shaders/                # WebGL shaders
│   ├── package.json
│   └── tsconfig.json
│
├── vscode-libvt/                   # ❌ NEW: VSCode extension
│   ├── src/
│   │   ├── extension.ts            # Extension entry
│   │   ├── terminal.ts             # Terminal provider
│   │   ├── pty.ts                  # PTY management
│   │   └── config.ts               # Configuration
│   ├── package.json
│   └── tsconfig.json
│
└── docs/
    ├── LIBVT_GAP_ANALYSIS.md       # ✅ Gap analysis
    ├── LIBVT_VS_LIBVTE_COMPARISON.md # ✅ libvte comparison
    ├── LIBVT_AS_XTERMJS_REPLACEMENT.md # ✅ xterm.js analysis
    └── IMPLEMENTATION_PLAN.md      # ✅ This document
```

---

## Phase 1: Core xterm.js API Compatibility
**Duration**: 8-10 weeks
**Priority**: 🔴 CRITICAL
**Dependencies**: None (builds on current foundation)

### 1.1 Event System (Week 1-2)

**Goal**: Implement xterm.js event model (onData, onBinary, onKey, etc.)

#### Tasks

**1.1.1 Expand Event Types** (2 days)
```rust
// libvt/src/events.rs
use crate::input::{KeyCode, KeyModifiers, MouseEvent};

#[derive(Debug, Clone)]
pub enum TerminalEvent {
    // ✅ Already exists
    Bell,
    TitleChanged(String),

    // ❌ NEW: Input events
    Data(Vec<u8>),              // User typed data
    Binary(Vec<u8>),            // Binary mode data
    Key {                        // Raw key event
        key: KeyCode,
        mods: KeyModifiers,
        text: String,
    },

    // ❌ NEW: State change events
    Resize { cols: u16, rows: u16 },
    CursorMove { col: u16, row: u16 },
    Scroll { position: usize },
    SelectionChange,
    LineFeed,
    WriteParsed,                // After write() completes

    // ❌ NEW: Buffer events
    BufferChange { active: BufferType },

    // ❌ NEW: Mode events
    ModeChange(TerminalMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    Normal,
    Alternate,
}
```

**1.1.2 Event Subscription API** (1 day)
```rust
// libvt/src/terminal.rs
impl Terminal {
    /// Subscribe to specific event types (filtering)
    pub fn subscribe_filtered<F>(
        &mut self,
        filter: EventFilter,
        callback: F,
    ) -> EventSubscriptionId
    where
        F: Fn(&TerminalEvent) -> bool + 'static,
    {
        // Implementation
    }

    /// Emit event to all subscribers
    fn emit_event(&mut self, event: TerminalEvent) {
        self.events.push(event.clone());
        for subscriber in &mut self.subscribers {
            subscriber.on_event(&event);
        }
    }
}
```

**1.1.3 Input Event Generation** (3 days)
```rust
impl Terminal {
    /// Handle user input and generate Data event
    pub fn handle_input(&mut self, data: &str) {
        self.emit_event(TerminalEvent::Data(data.as_bytes().to_vec()));
    }

    /// Handle key press and generate Key event
    pub fn handle_key(&mut self, key: KeyCode, mods: KeyModifiers) {
        let bytes = self.encode_key(key, mods);
        let text = String::from_utf8_lossy(&bytes).to_string();

        self.emit_event(TerminalEvent::Key {
            key,
            mods,
            text: text.clone(),
        });

        // Also emit Data for compatibility
        self.emit_event(TerminalEvent::Data(bytes));
    }
}
```

**Deliverables**:
- ✅ 8 new event types
- ✅ Event filtering API
- ✅ Input event generation
- ✅ 20+ tests for event system

---

### 1.2 Alternate Screen Buffer (Week 3)

**Goal**: Implement CSI ?1049h/l for vim, less, tmux

#### Tasks

**1.2.1 Screen Buffer Abstraction** (2 days)
```rust
// libvt/src/buffer.rs - NEW FILE
use crate::screen::Screen;
use crate::cursor::Cursor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    Normal,
    Alternate,
}

pub struct BufferSet {
    normal: Screen,
    alternate: Screen,
    active: BufferType,

    // Cursor state per buffer
    normal_cursor: Cursor,
    alternate_cursor: Cursor,
}

impl BufferSet {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            normal: Screen::new(cols, rows),
            alternate: Screen::new(cols, rows),
            active: BufferType::Normal,
            normal_cursor: Cursor::new(),
            alternate_cursor: Cursor::new(),
        }
    }

    pub fn active(&self) -> &Screen {
        match self.active {
            BufferType::Normal => &self.normal,
            BufferType::Alternate => &self.alternate,
        }
    }

    pub fn active_mut(&mut self) -> &mut Screen {
        match self.active {
            BufferType::Normal => &mut self.normal,
            BufferType::Alternate => &mut self.alternate,
        }
    }

    pub fn switch_to(&mut self, buffer_type: BufferType) {
        if self.active == buffer_type {
            return;
        }

        // Save current cursor
        match self.active {
            BufferType::Normal => self.normal_cursor = self.get_active_cursor().clone(),
            BufferType::Alternate => self.alternate_cursor = self.get_active_cursor().clone(),
        }

        self.active = buffer_type;

        // Restore cursor for new buffer
        // (handled by caller)
    }

    fn get_active_cursor(&self) -> &Cursor {
        match self.active {
            BufferType::Normal => &self.normal_cursor,
            BufferType::Alternate => &self.alternate_cursor,
        }
    }
}
```

**1.2.2 Terminal Integration** (2 days)
```rust
// libvt/src/terminal.rs
pub struct Terminal {
    // OLD: screen: Screen,
    // NEW:
    buffers: BufferSet,

    // ... rest of fields
}

impl Terminal {
    pub fn switch_to_alternate_screen(&mut self) {
        self.buffers.switch_to(BufferType::Alternate);
        self.buffers.active_mut().clear();
        self.emit_event(TerminalEvent::BufferChange {
            active: BufferType::Alternate
        });
    }

    pub fn switch_to_normal_screen(&mut self) {
        self.buffers.switch_to(BufferType::Normal);
        self.emit_event(TerminalEvent::BufferChange {
            active: BufferType::Normal
        });
    }

    pub fn is_alternate_screen(&self) -> bool {
        self.buffers.active == BufferType::Alternate
    }

    pub fn buffer(&self) -> &Screen {
        self.buffers.active()
    }

    pub fn buffer_mut(&mut self) -> &mut Screen {
        self.buffers.active_mut()
    }
}
```

**1.2.3 CSI Sequence Handlers** (1 day)
```rust
// libvt/src/terminal.rs - in perform_action()
Action::Csi(csi) => {
    match csi.params() {
        // CSI ?1049h - Switch to alternate screen
        [1049] if csi.intermediates() == &[b'?'] && csi.final_byte() == b'h' => {
            self.switch_to_alternate_screen();
        }
        // CSI ?1049l - Switch to normal screen
        [1049] if csi.intermediates() == &[b'?'] && csi.final_byte() == b'l' => {
            self.switch_to_normal_screen();
        }
        // ... other CSI sequences
    }
}
```

**Deliverables**:
- ✅ BufferSet abstraction
- ✅ Screen switching logic
- ✅ CSI ?1049h/l handlers
- ✅ vim workflow test (open, edit, quit)
- ✅ 15+ tests

---

### 1.3 Enhanced Selection (Week 4-5)

**Goal**: Block selection, word/line selection, getSelectionPosition()

#### Tasks

**1.3.1 Selection Types** (2 days)
```rust
// libvt/src/selection.rs - NEW FILE
use crate::screen::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Stream,      // Normal text selection (default)
    Block,       // Rectangular/column selection
    Line,        // Whole lines
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub start: Position,
    pub end: Position,
    pub mode: SelectionMode,
}

impl Selection {
    pub fn new(start: Position, end: Position) -> Self {
        Self {
            start,
            end,
            mode: SelectionMode::Stream,
        }
    }

    pub fn with_mode(start: Position, end: Position, mode: SelectionMode) -> Self {
        Self { start, end, mode }
    }

    /// Normalize so start <= end
    pub fn normalize(&mut self) {
        if self.start.row > self.end.row
            || (self.start.row == self.end.row && self.start.col > self.end.col) {
            std::mem::swap(&mut self.start, &mut self.end);
        }
    }

    /// Check if position is within selection
    pub fn contains(&self, pos: Position) -> bool {
        let mut normalized = self.clone();
        normalized.normalize();

        match self.mode {
            SelectionMode::Stream => {
                // Standard stream selection
                if pos.row < normalized.start.row || pos.row > normalized.end.row {
                    return false;
                }
                if pos.row == normalized.start.row && pos.col < normalized.start.col {
                    return false;
                }
                if pos.row == normalized.end.row && pos.col > normalized.end.col {
                    return false;
                }
                true
            }
            SelectionMode::Block => {
                // Rectangular selection
                pos.row >= normalized.start.row
                    && pos.row <= normalized.end.row
                    && pos.col >= normalized.start.col
                    && pos.col <= normalized.end.col
            }
            SelectionMode::Line => {
                pos.row >= normalized.start.row && pos.row <= normalized.end.row
            }
        }
    }
}
```

**1.3.2 Selection API** (3 days)
```rust
// libvt/src/terminal.rs
impl Terminal {
    /// Select specific range (xterm.js compatible)
    pub fn select(&mut self, col: u16, row: u16, length: u16) {
        let start = Position { col, row };
        let end = Position {
            col: col + length - 1,
            row
        };
        self.set_selection(start, end);
    }

    /// Select all visible content
    pub fn select_all(&mut self) {
        let (cols, rows) = self.size();
        let start = Position { col: 0, row: 0 };
        let end = Position {
            col: cols - 1,
            row: rows - 1
        };
        self.set_selection(start, end);
        self.emit_event(TerminalEvent::SelectionChange);
    }

    /// Select entire lines
    pub fn select_lines(&mut self, start_row: u16, end_row: u16) {
        let start = Position { col: 0, row: start_row };
        let end = Position { col: 0, row: end_row };
        self.selection = Some(Selection::with_mode(
            start,
            end,
            SelectionMode::Line
        ));
        self.emit_event(TerminalEvent::SelectionChange);
    }

    /// Check if has selection
    pub fn has_selection(&self) -> bool {
        self.selection.is_some()
    }

    /// Get selection position
    pub fn get_selection_position(&self) -> Option<(Position, Position)> {
        self.selection.as_ref().map(|s| {
            let mut sel = s.clone();
            sel.normalize();
            (sel.start, sel.end)
        })
    }

    /// Extend selection (Shift+Click simulation)
    pub fn extend_selection(&mut self, pos: Position) {
        if let Some(sel) = &mut self.selection {
            // Determine which endpoint to move
            let dist_to_start = self.distance(pos, sel.start);
            let dist_to_end = self.distance(pos, sel.end);

            if dist_to_start < dist_to_end {
                sel.start = pos;
            } else {
                sel.end = pos;
            }

            self.emit_event(TerminalEvent::SelectionChange);
        }
    }

    fn distance(&self, a: Position, b: Position) -> usize {
        let row_diff = (a.row as isize - b.row as isize).unsigned_abs();
        let col_diff = (a.col as isize - b.col as isize).unsigned_abs();
        row_diff + col_diff
    }
}
```

**1.3.3 Text Extraction** (2 days)
```rust
impl Terminal {
    /// Get selected text
    pub fn selected_text(&self) -> Option<String> {
        let sel = self.selection.as_ref()?;
        let mut sel = sel.clone();
        sel.normalize();

        let mut text = String::new();

        match sel.mode {
            SelectionMode::Stream => {
                for row in sel.start.row..=sel.end.row {
                    if let Some(line) = self.buffer().get_line(row as usize) {
                        let start_col = if row == sel.start.row {
                            sel.start.col
                        } else {
                            0
                        };
                        let end_col = if row == sel.end.row {
                            sel.end.col + 1
                        } else {
                            line.width() as u16
                        };

                        text.push_str(&line.get_text_range(
                            start_col as usize,
                            end_col as usize
                        ));

                        if row < sel.end.row && !line.is_wrapped() {
                            text.push('\n');
                        }
                    }
                }
            }
            SelectionMode::Block => {
                for row in sel.start.row..=sel.end.row {
                    if let Some(line) = self.buffer().get_line(row as usize) {
                        text.push_str(&line.get_text_range(
                            sel.start.col as usize,
                            (sel.end.col + 1) as usize
                        ));
                        text.push('\n');
                    }
                }
            }
            SelectionMode::Line => {
                for row in sel.start.row..=sel.end.row {
                    if let Some(line) = self.buffer().get_line(row as usize) {
                        text.push_str(&line.to_string());
                        text.push('\n');
                    }
                }
            }
        }

        Some(text)
    }
}
```

**Deliverables**:
- ✅ SelectionMode enum (Stream, Block, Line)
- ✅ Enhanced selection API (7 methods)
- ✅ Text extraction with proper line wrapping
- ✅ 25+ tests for selection edge cases

---

### 1.4 Scrolling API (Week 6)

**Goal**: scrollLines(), scrollPages(), scrollToTop(), scrollToBottom()

#### Tasks

**1.4.1 Viewport Management** (2 days)
```rust
// libvt/src/scrollback.rs - NEW FILE
pub struct ScrollbackManager {
    /// Current scroll offset (0 = at bottom)
    scroll_offset: usize,

    /// Total scrollback lines
    max_scrollback: usize,

    /// Viewport height
    viewport_rows: usize,
}

impl ScrollbackManager {
    pub fn new(max_scrollback: usize, viewport_rows: usize) -> Self {
        Self {
            scroll_offset: 0,
            max_scrollback,
            viewport_rows,
        }
    }

    pub fn scroll_by(&mut self, delta: isize, total_lines: usize) -> bool {
        let old_offset = self.scroll_offset;

        let new_offset = if delta > 0 {
            // Scroll up (into history)
            (self.scroll_offset + delta as usize)
                .min(total_lines.saturating_sub(self.viewport_rows))
        } else {
            // Scroll down (toward bottom)
            self.scroll_offset.saturating_sub((-delta) as usize)
        };

        self.scroll_offset = new_offset;
        self.scroll_offset != old_offset // Changed?
    }

    pub fn scroll_to_top(&mut self, total_lines: usize) -> bool {
        let old = self.scroll_offset;
        self.scroll_offset = total_lines.saturating_sub(self.viewport_rows);
        self.scroll_offset != old
    }

    pub fn scroll_to_bottom(&mut self) -> bool {
        let old = self.scroll_offset;
        self.scroll_offset = 0;
        old != 0
    }

    pub fn scroll_to_line(&mut self, line: usize, total_lines: usize) -> bool {
        let old = self.scroll_offset;
        self.scroll_offset = line.min(total_lines.saturating_sub(self.viewport_rows));
        self.scroll_offset != old
    }

    pub fn offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn is_at_bottom(&self) -> bool {
        self.scroll_offset == 0
    }
}
```

**1.4.2 Terminal Scrolling API** (2 days)
```rust
// libvt/src/terminal.rs
impl Terminal {
    pub fn scroll_lines(&mut self, amount: isize) {
        let total = self.scrollback.len() + self.rows as usize;

        if self.scrollback_mgr.scroll_by(amount, total) {
            self.emit_event(TerminalEvent::Scroll {
                position: self.scrollback_mgr.offset(),
            });
        }
    }

    pub fn scroll_pages(&mut self, page_count: isize) {
        let lines = page_count * (self.rows as isize);
        self.scroll_lines(lines);
    }

    pub fn scroll_to_top(&mut self) {
        let total = self.scrollback.len() + self.rows as usize;

        if self.scrollback_mgr.scroll_to_top(total) {
            self.emit_event(TerminalEvent::Scroll {
                position: self.scrollback_mgr.offset(),
            });
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        if self.scrollback_mgr.scroll_to_bottom() {
            self.emit_event(TerminalEvent::Scroll {
                position: 0,
            });
        }
    }

    pub fn scroll_to_line(&mut self, line: usize) {
        let total = self.scrollback.len() + self.rows as usize;

        if self.scrollback_mgr.scroll_to_line(line, total) {
            self.emit_event(TerminalEvent::Scroll {
                position: self.scrollback_mgr.offset(),
            });
        }
    }

    pub fn scroll_offset(&self) -> usize {
        self.scrollback_mgr.offset()
    }
}
```

**Deliverables**:
- ✅ ScrollbackManager abstraction
- ✅ 5 scrolling methods
- ✅ Scroll event emission
- ✅ 15+ tests

---

### 1.5 Input Handling (Week 7)

**Goal**: paste(), input(), binary mode, bracketed paste

#### Tasks

**1.5.1 Paste Support** (2 days)
```rust
// libvt/src/terminal.rs
impl Terminal {
    /// Paste text (handles bracketed paste mode)
    pub fn paste(&mut self, data: &str) {
        if self.modes.bracketed_paste {
            // Wrap in bracketed paste sequences
            self.write(b"\x1b[200~");
            self.write(data.as_bytes());
            self.write(b"\x1b[201~");
        } else {
            self.write(data.as_bytes());
        }
    }

    /// High-level input (can be user or programmatic)
    pub fn input(&mut self, data: &str, was_user_input: bool) {
        if was_user_input {
            // Emit Data event for listeners
            self.emit_event(TerminalEvent::Data(data.as_bytes().to_vec()));
        }

        // Could transform data here (e.g., local echo)
        // For now, just pass through
    }
}
```

**1.5.2 Binary Mode** (1 day)
```rust
// libvt/src/modes.rs - NEW FILE
#[derive(Debug, Clone, Default)]
pub struct TerminalModes {
    pub application_cursor_keys: bool,
    pub application_keypad: bool,
    pub bracketed_paste: bool,
    pub insert_mode: bool,
    pub auto_wrap: bool,
    pub origin_mode: bool,
    pub reverse_wraparound: bool,
    pub send_focus: bool,
    pub mouse_tracking: MouseTrackingMode,
    pub binary_mode: bool,  // NEW
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseTrackingMode {
    Off,
    X10,
    VT200,
    VT200Highlight,
    ButtonEvent,
    AnyEvent,
}

impl Default for MouseTrackingMode {
    fn default() -> Self {
        Self::Off
    }
}
```

**1.5.3 Custom Event Handlers** (1 day)
```rust
// libvt/src/terminal.rs
pub type CustomKeyHandler = Box<dyn Fn(KeyCode, KeyModifiers) -> bool>;
pub type CustomWheelHandler = Box<dyn Fn(i32) -> bool>;

impl Terminal {
    pub fn attach_custom_key_handler(&mut self, handler: CustomKeyHandler) {
        self.custom_key_handler = Some(handler);
    }

    pub fn attach_custom_wheel_handler(&mut self, handler: CustomWheelHandler) {
        self.custom_wheel_handler = Some(handler);
    }

    fn handle_key_internal(&mut self, key: KeyCode, mods: KeyModifiers) {
        // Check custom handler first
        if let Some(ref handler) = self.custom_key_handler {
            if handler(key, mods) {
                return; // Handler consumed it
            }
        }

        // Default handling
        self.handle_key(key, mods);
    }
}
```

**Deliverables**:
- ✅ paste() method with bracketed paste
- ✅ input() method
- ✅ Binary mode support
- ✅ Custom handler API
- ✅ 10+ tests

---

### 1.6 Markers & Decorations (Week 8)

**Goal**: registerMarker(), marker tracking, decorations API

#### Tasks

**1.6.1 Marker System** (3 days)
```rust
// libvt/src/markers.rs - NEW FILE
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_MARKER_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarkerId(usize);

impl MarkerId {
    fn new() -> Self {
        Self(NEXT_MARKER_ID.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Debug, Clone)]
pub struct Marker {
    id: MarkerId,
    /// Line in scrollback (absolute, not viewport-relative)
    line: usize,
    /// Whether marker is still valid
    disposed: bool,
}

impl Marker {
    pub fn new(line: usize) -> Self {
        Self {
            id: MarkerId::new(),
            line,
            disposed: false,
        }
    }

    pub fn id(&self) -> MarkerId {
        self.id
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn dispose(&mut self) {
        self.disposed = true;
    }

    pub fn is_disposed(&self) -> bool {
        self.disposed
    }
}

pub struct MarkerManager {
    markers: Vec<Marker>,
}

impl MarkerManager {
    pub fn new() -> Self {
        Self {
            markers: Vec::new(),
        }
    }

    pub fn add_marker(&mut self, line: usize) -> MarkerId {
        let marker = Marker::new(line);
        let id = marker.id();
        self.markers.push(marker);
        id
    }

    pub fn remove_marker(&mut self, id: MarkerId) {
        if let Some(marker) = self.markers.iter_mut().find(|m| m.id() == id) {
            marker.dispose();
        }
    }

    pub fn get_marker(&self, id: MarkerId) -> Option<&Marker> {
        self.markers.iter().find(|m| m.id() == id && !m.is_disposed())
    }

    pub fn active_markers(&self) -> impl Iterator<Item = &Marker> {
        self.markers.iter().filter(|m| !m.is_disposed())
    }

    /// Adjust marker positions when lines are added/removed
    pub fn adjust_for_scroll(&mut self, lines_added: usize) {
        for marker in &mut self.markers {
            if !marker.disposed {
                marker.line += lines_added;
            }
        }
    }

    pub fn cleanup_disposed(&mut self) {
        self.markers.retain(|m| !m.disposed);
    }
}
```

**1.6.2 Terminal Integration** (1 day)
```rust
// libvt/src/terminal.rs
impl Terminal {
    pub fn register_marker(&mut self, cursor_y_offset: Option<i32>) -> MarkerId {
        let offset = cursor_y_offset.unwrap_or(0);
        let cursor_line = self.cursor_position().1 as i32;
        let line = (cursor_line + offset).max(0) as usize;

        // Convert viewport line to absolute scrollback line
        let absolute_line = self.scrollback.len() + line;

        self.markers.add_marker(absolute_line)
    }

    pub fn markers(&self) -> Vec<&Marker> {
        self.markers.active_markers().collect()
    }
}
```

**Deliverables**:
- ✅ Marker system with IDs
- ✅ registerMarker() API
- ✅ Marker tracking during scroll
- ✅ Cleanup mechanism
- ✅ 15+ tests

---

### 1.7 Parser Extension API (Week 9-10)

**Goal**: Custom escape sequence handlers (registerCsiHandler, etc.)

#### Tasks

**1.7.1 Handler Registry** (3 days)
```rust
// libvt/src/parser/handlers.rs - NEW FILE
use crate::parser::Action;
use std::collections::HashMap;

pub type CsiHandler = Box<dyn Fn(&[u16]) -> bool + Send>;
pub type OscHandler = Box<dyn Fn(&str) -> bool + Send>;
pub type EscHandler = Box<dyn Fn() -> bool + Send>;
pub type DcsHandler = Box<dyn Fn(&str, &[u16]) -> bool + Send>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionIdentifier {
    pub prefix: Option<u8>,
    pub intermediates: Option<u8>,
    pub final_byte: u8,
}

impl FunctionIdentifier {
    pub fn new(final_byte: u8) -> Self {
        Self {
            prefix: None,
            intermediates: None,
            final_byte,
        }
    }

    pub fn with_prefix(mut self, prefix: u8) -> Self {
        self.prefix = Some(prefix);
        self
    }

    pub fn with_intermediate(mut self, intermediate: u8) -> Self {
        self.intermediates = Some(intermediate);
        self
    }
}

pub struct HandlerRegistry {
    csi_handlers: HashMap<FunctionIdentifier, Vec<CsiHandler>>,
    osc_handlers: HashMap<u16, Vec<OscHandler>>,
    esc_handlers: HashMap<FunctionIdentifier, Vec<EscHandler>>,
    dcs_handlers: HashMap<FunctionIdentifier, Vec<DcsHandler>>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            csi_handlers: HashMap::new(),
            osc_handlers: HashMap::new(),
            esc_handlers: HashMap::new(),
            dcs_handlers: HashMap::new(),
        }
    }

    pub fn add_csi_handler(&mut self, id: FunctionIdentifier, handler: CsiHandler) {
        self.csi_handlers.entry(id).or_insert_with(Vec::new).push(handler);
    }

    pub fn add_osc_handler(&mut self, ident: u16, handler: OscHandler) {
        self.osc_handlers.entry(ident).or_insert_with(Vec::new).push(handler);
    }

    pub fn invoke_csi(&self, id: FunctionIdentifier, params: &[u16]) -> bool {
        if let Some(handlers) = self.csi_handlers.get(&id) {
            for handler in handlers {
                if handler(params) {
                    return true; // Handler consumed it
                }
            }
        }
        false
    }

    pub fn invoke_osc(&self, ident: u16, data: &str) -> bool {
        if let Some(handlers) = self.osc_handlers.get(&ident) {
            for handler in handlers {
                if handler(data) {
                    return true;
                }
            }
        }
        false
    }
}
```

**1.7.2 Parser Integration** (2 days)
```rust
// libvt/src/parser/mod.rs
use crate::parser::handlers::HandlerRegistry;

pub struct Parser {
    // ... existing fields
    handlers: HandlerRegistry,
}

impl Parser {
    pub fn register_csi_handler(&mut self, id: FunctionIdentifier, handler: CsiHandler) {
        self.handlers.add_csi_handler(id, handler);
    }

    pub fn register_osc_handler(&mut self, ident: u16, handler: OscHandler) {
        self.handlers.add_osc_handler(ident, handler);
    }

    // In parse loop, check custom handlers first
    fn handle_csi(&mut self, csi: CsiSequence) -> Option<Action> {
        let id = FunctionIdentifier {
            prefix: csi.prefix,
            intermediates: csi.intermediates.first().copied(),
            final_byte: csi.final_byte,
        };

        if self.handlers.invoke_csi(id, csi.params) {
            return None; // Custom handler consumed it
        }

        // Default handling
        Some(Action::Csi(csi))
    }
}
```

**1.7.3 Terminal API** (1 day)
```rust
// libvt/src/terminal.rs
impl Terminal {
    pub fn register_csi_handler<F>(&mut self, id: FunctionIdentifier, handler: F)
    where
        F: Fn(&[u16]) -> bool + Send + 'static,
    {
        self.parser.register_csi_handler(id, Box::new(handler));
    }

    pub fn register_osc_handler<F>(&mut self, ident: u16, handler: F)
    where
        F: Fn(&str) -> bool + Send + 'static,
    {
        self.parser.register_osc_handler(ident, Box::new(handler));
    }
}
```

**Deliverables**:
- ✅ Handler registry system
- ✅ 4 handler types (CSI, OSC, ESC, DCS)
- ✅ Parser integration
- ✅ Public API for registration
- ✅ 20+ tests with custom handlers

---

## Phase 1 Summary

**Deliverables**:
- ✅ Event system (8 event types)
- ✅ Alternate screen buffer (vim/less work)
- ✅ Enhanced selection (block, word, line)
- ✅ Scrolling API (5 methods)
- ✅ Input handling (paste, binary, custom handlers)
- ✅ Marker system
- ✅ Parser extension API

**Testing**:
- ✅ 120+ new unit tests
- ✅ vim workflow test
- ✅ Selection edge cases
- ✅ Custom handler tests

**Code**: +2,000 LOC (total: ~5,200 LOC)

---

## Phase 2: Buffer Access API
**Duration**: 3-4 weeks
**Priority**: 🟡 HIGH
**Dependencies**: Phase 1 (alternate screen)

### 2.1 IBuffer Interface (Week 11-12)

**Goal**: buffer.active, buffer.normal, buffer.alternate, buffer properties

#### Tasks

**2.1.1 Buffer Abstraction** (3 days)
```rust
// libvt/src/buffer.rs (expand existing)
pub struct Buffer<'a> {
    terminal: &'a Terminal,
    buffer_type: BufferType,
}

impl<'a> Buffer<'a> {
    pub fn buffer_type(&self) -> BufferType {
        self.buffer_type
    }

    pub fn cursor_x(&self) -> u16 {
        self.terminal.cursor_position().0
    }

    pub fn cursor_y(&self) -> u16 {
        // Viewport-relative
        self.terminal.cursor_position().1
    }

    pub fn viewport_y(&self) -> usize {
        // Top of viewport in scrollback
        self.terminal.scroll_offset()
    }

    pub fn base_y(&self) -> usize {
        // Top of buffer (always 0 for alt, scrollback.len() for normal)
        match self.buffer_type {
            BufferType::Normal => self.terminal.scrollback.len(),
            BufferType::Alternate => 0,
        }
    }

    pub fn length(&self) -> usize {
        // Total lines including scrollback
        match self.buffer_type {
            BufferType::Normal => {
                self.terminal.scrollback.len() + self.terminal.rows as usize
            }
            BufferType::Alternate => self.terminal.rows as usize,
        }
    }

    pub fn get_line(&self, y: usize) -> Option<BufferLine> {
        let screen = match self.buffer_type {
            BufferType::Normal => &self.terminal.buffers.normal,
            BufferType::Alternate => &self.terminal.buffers.alternate,
        };

        // y is absolute (includes scrollback)
        if y < self.terminal.scrollback.len() {
            self.terminal.scrollback.get(y).map(|line| BufferLine {
                line: line.clone(),
                row: y,
            })
        } else {
            let viewport_y = y - self.terminal.scrollback.len();
            screen.get_line(viewport_y).map(|line| BufferLine {
                line: line.clone(),
                row: y,
            })
        }
    }

    pub fn get_null_cell(&self) -> BufferCell {
        BufferCell {
            cell: Cell::blank(),
        }
    }
}
```

**2.1.2 Terminal API** (2 days)
```rust
// libvt/src/terminal.rs
impl Terminal {
    pub fn buffer(&self) -> Buffer {
        Buffer {
            terminal: self,
            buffer_type: self.buffers.active,
        }
    }

    pub fn normal_buffer(&self) -> Buffer {
        Buffer {
            terminal: self,
            buffer_type: BufferType::Normal,
        }
    }

    pub fn alternate_buffer(&self) -> Buffer {
        Buffer {
            terminal: self,
            buffer_type: BufferType::Alternate,
        }
    }
}
```

**Deliverables**:
- ✅ Buffer abstraction with 8 properties
- ✅ buffer, normal_buffer, alternate_buffer methods
- ✅ 20+ tests

---

### 2.2 IBufferLine & IBufferCell (Week 13-14)

**Goal**: Complete line and cell API for buffer access

#### Tasks

**2.2.1 BufferLine API** (3 days)
```rust
// libvt/src/buffer.rs
#[derive(Debug, Clone)]
pub struct BufferLine {
    line: Line,
    row: usize,
}

impl BufferLine {
    pub fn is_wrapped(&self) -> bool {
        self.line.is_wrapped()
    }

    pub fn length(&self) -> usize {
        self.line.width()
    }

    pub fn get_cell(&self, x: usize) -> Option<BufferCell> {
        self.line.get_cell(x).map(|cell| BufferCell {
            cell: cell.clone(),
        })
    }

    pub fn translate_to_string(&self, trim_right: bool, start_col: Option<usize>, end_col: Option<usize>) -> String {
        let start = start_col.unwrap_or(0);
        let end = end_col.unwrap_or(self.line.width());

        let mut text = self.line.get_text_range(start, end);

        if trim_right {
            text = text.trim_end().to_string();
        }

        text
    }
}
```

**2.2.2 BufferCell API** (2 days)
```rust
// libvt/src/buffer.rs
#[derive(Debug, Clone)]
pub struct BufferCell {
    cell: Cell,
}

impl BufferCell {
    pub fn get_chars(&self) -> String {
        self.cell.text().to_string()
    }

    pub fn get_width(&self) -> u8 {
        self.cell.width()
    }

    pub fn get_code(&self) -> u32 {
        self.cell.text().chars().next().unwrap_or(' ') as u32
    }

    pub fn is_bold(&self) -> bool {
        self.cell.attrs().bold
    }

    pub fn is_italic(&self) -> bool {
        self.cell.attrs().italic
    }

    pub fn is_dim(&self) -> bool {
        self.cell.attrs().dim
    }

    pub fn is_underline(&self) -> bool {
        self.cell.attrs().underline != UnderlineStyle::None
    }

    pub fn is_blink(&self) -> bool {
        self.cell.attrs().blink
    }

    pub fn is_inverse(&self) -> bool {
        self.cell.attrs().reverse
    }

    pub fn is_invisible(&self) -> bool {
        self.cell.attrs().hidden
    }

    pub fn is_strikethrough(&self) -> bool {
        self.cell.attrs().strikethrough
    }

    pub fn get_fg_color_mode(&self) -> ColorMode {
        match &self.cell.attrs().foreground {
            ColorSpec::Default => ColorMode::Default,
            ColorSpec::Ansi(_) => ColorMode::Palette,
            ColorSpec::Rgb(_) => ColorMode::Rgb,
        }
    }

    pub fn get_bg_color_mode(&self) -> ColorMode {
        match &self.cell.attrs().background {
            ColorSpec::Default => ColorMode::Default,
            ColorSpec::Ansi(_) => ColorMode::Palette,
            ColorSpec::Rgb(_) => ColorMode::Rgb,
        }
    }

    pub fn get_fg_color(&self) -> u32 {
        match &self.cell.attrs().foreground {
            ColorSpec::Default => 0,
            ColorSpec::Ansi(idx) => *idx as u32,
            ColorSpec::Rgb(rgb) => {
                ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | (rgb.b as u32)
            }
        }
    }

    pub fn get_bg_color(&self) -> u32 {
        match &self.cell.attrs().background {
            ColorSpec::Default => 0,
            ColorSpec::Ansi(idx) => *idx as u32,
            ColorSpec::Rgb(rgb) => {
                ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | (rgb.b as u32)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Default = 0,
    Palette = 1,
    Rgb = 2,
}
```

**Deliverables**:
- ✅ BufferLine with 4 methods
- ✅ BufferCell with 16 methods
- ✅ ColorMode enum
- ✅ 30+ tests for buffer access

---

## Phase 2 Summary

**Deliverables**:
- ✅ IBuffer interface (9 properties/methods)
- ✅ IBufferLine interface (4 methods)
- ✅ IBufferCell interface (16 methods)
- ✅ Complete buffer access API

**Testing**:
- ✅ 50+ buffer access tests
- ✅ Edge case handling

**Code**: +800 LOC (total: ~6,000 LOC)

---

## Phase 3: Advanced Features
**Duration**: 6-8 weeks
**Priority**: 🟠 MEDIUM
**Dependencies**: Phase 1-2

### 3.1 Unicode Handling (Week 15-16)

**Goal**: Unicode version API, grapheme clusters, proper width calculation

#### Tasks

**3.1.1 Unicode API** (3 days)
```rust
// libvt/src/unicode.rs - NEW FILE
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnicodeVersion {
    V6 = 6,
    V11 = 11,
    V14 = 14,
    V15 = 15,
}

impl Default for UnicodeVersion {
    fn default() -> Self {
        Self::V15
    }
}

pub struct UnicodeHandling {
    active_version: UnicodeVersion,
    available_versions: Vec<UnicodeVersion>,
}

impl UnicodeHandling {
    pub fn new() -> Self {
        Self {
            active_version: UnicodeVersion::default(),
            available_versions: vec![
                UnicodeVersion::V6,
                UnicodeVersion::V11,
                UnicodeVersion::V14,
                UnicodeVersion::V15,
            ],
        }
    }

    pub fn active_version(&self) -> UnicodeVersion {
        self.active_version
    }

    pub fn set_active_version(&mut self, version: UnicodeVersion) {
        self.active_version = version;
    }

    pub fn versions(&self) -> &[UnicodeVersion] {
        &self.available_versions
    }

    pub fn char_width(&self, ch: char, prev_ch: Option<char>) -> u8 {
        // Use unicode-width crate with version-specific tables
        // For now, simplified:
        match unicode_width::UnicodeWidthChar::width(ch) {
            Some(0) => 0, // Combining characters
            Some(1) => 1,
            Some(2) => 2, // CJK, emoji
            Some(w) => w as u8,
            None => 1,
        }
    }
}
```

**3.1.2 Grapheme Cluster Support** (4 days)
```rust
// Add dependency: unicode-segmentation = "1.10"

use unicode_segmentation::UnicodeSegmentation;

impl Terminal {
    /// Write text with proper grapheme cluster handling
    fn write_text(&mut self, text: &str) {
        for grapheme in text.graphemes(true) {
            let width = self.unicode.char_width(
                grapheme.chars().next().unwrap(),
                None
            );

            // Handle wide characters (emoji, CJK)
            if width == 2 {
                // Occupies 2 cells
                self.write_wide_grapheme(grapheme);
            } else {
                self.write_grapheme(grapheme);
            }
        }
    }

    fn write_wide_grapheme(&mut self, grapheme: &str) {
        let (col, row) = self.cursor_position();

        if col >= self.cols - 1 {
            // Not enough space, wrap
            self.line_feed();
        }

        // Write to current cell
        if let Some(cell) = self.buffer_mut().get_cell_mut(col as usize, row as usize) {
            cell.set_text(grapheme.to_string());
        }

        // Mark next cell as continuation
        if let Some(cell) = self.buffer_mut().get_cell_mut((col + 1) as usize, row as usize) {
            cell.set_text(String::new()); // Empty (wide char continuation)
        }

        self.cursor.col += 2;
    }
}
```

**Deliverables**:
- ✅ UnicodeHandling API
- ✅ Grapheme cluster support
- ✅ Wide character handling
- ✅ 25+ Unicode tests (emoji, CJK, combining chars)

---

### 3.2 Modes API (Week 17-18)

**Goal**: Expose all terminal modes via clean API

#### Tasks

**3.2.1 Modes Structure** (2 days)
```rust
// libvt/src/modes.rs (expand existing)
#[derive(Debug, Clone)]
pub struct TerminalModes {
    // Cursor
    pub application_cursor_keys: bool,  // DECCKM

    // Keypad
    pub application_keypad: bool,       // DECKPAM

    // Input
    pub bracketed_paste: bool,          // 2004
    pub send_focus: bool,               // 1004

    // Display
    pub insert_mode: bool,              // IRM
    pub auto_wrap: bool,                // DECAWM (7)
    pub origin_mode: bool,              // DECOM (6)
    pub reverse_wraparound: bool,       // 45

    // Mouse
    pub mouse_tracking: MouseTrackingMode,

    // Screen
    pub alternate_screen: bool,         // 1049

    // Misc
    pub show_cursor: bool,              // DECTCEM (25)
}

impl TerminalModes {
    pub fn new() -> Self {
        Self {
            application_cursor_keys: false,
            application_keypad: false,
            bracketed_paste: false,
            send_focus: false,
            insert_mode: false,
            auto_wrap: true,
            origin_mode: false,
            reverse_wraparound: false,
            mouse_tracking: MouseTrackingMode::Off,
            alternate_screen: false,
            show_cursor: true,
        }
    }

    pub fn set_mode(&mut self, mode: u16, enable: bool) {
        match mode {
            1 => self.application_cursor_keys = enable,  // DECCKM
            2 => {}, // DECANM (ANSI mode) - ignore
            3 => {}, // DECCOLM (132 column mode) - ignore for now
            4 => {}, // DECSCLM (smooth scroll) - ignore
            5 => {}, // DECSCNM (reverse video) - TODO
            6 => self.origin_mode = enable,               // DECOM
            7 => self.auto_wrap = enable,                 // DECAWM
            25 => self.show_cursor = enable,              // DECTCEM
            45 => self.reverse_wraparound = enable,
            1000 => self.mouse_tracking = if enable { MouseTrackingMode::X10 } else { MouseTrackingMode::Off },
            1002 => self.mouse_tracking = if enable { MouseTrackingMode::ButtonEvent } else { MouseTrackingMode::Off },
            1003 => self.mouse_tracking = if enable { MouseTrackingMode::AnyEvent } else { MouseTrackingMode::Off },
            1004 => self.send_focus = enable,
            1006 => {}, // SGR mouse mode - handled elsewhere
            1049 => self.alternate_screen = enable,
            2004 => self.bracketed_paste = enable,
            _ => {
                log::warn!("Unknown mode: {}", mode);
            }
        }
    }
}
```

**3.2.2 Mode Change Events** (1 day)
```rust
// Update events.rs
pub enum TerminalEvent {
    // ... existing events
    ModeChange {
        mode: u16,
        enabled: bool,
    },
}
```

**Deliverables**:
- ✅ Complete TerminalModes struct
- ✅ set_mode() implementation
- ✅ Mode change events
- ✅ 15+ mode tests

---

### 3.3 Link Provider API (Week 19-20)

**Goal**: Custom link detection and hyperlink handling

#### Tasks

**3.3.1 Link Detection** (3 days)
```rust
// libvt/src/links.rs - NEW FILE
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(
        r"(?i)(?:https?|ftp)://[^\s<>{}|\\^`\[\]]+[^\s<>{}|\\^`\[\].,;:?!()]"
    ).unwrap();

    static ref EMAIL_REGEX: Regex = Regex::new(
        r"(?i)[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}"
    ).unwrap();
}

#[derive(Debug, Clone)]
pub struct Link {
    pub text: String,
    pub start_col: u16,
    pub end_col: u16,
    pub row: u16,
    pub uri: Option<String>,
}

pub trait LinkProvider: Send + Sync {
    fn provide_links(&self, line: &str, row: u16) -> Vec<Link>;
}

pub struct DefaultLinkProvider;

impl LinkProvider for DefaultLinkProvider {
    fn provide_links(&self, line: &str, row: u16) -> Vec<Link> {
        let mut links = Vec::new();

        // Find URLs
        for mat in URL_REGEX.find_iter(line) {
            links.push(Link {
                text: mat.as_str().to_string(),
                start_col: mat.start() as u16,
                end_col: mat.end() as u16,
                row,
                uri: Some(mat.as_str().to_string()),
            });
        }

        // Find emails
        for mat in EMAIL_REGEX.find_iter(line) {
            links.push(Link {
                text: mat.as_str().to_string(),
                start_col: mat.start() as u16,
                end_col: mat.end() as u16,
                row,
                uri: Some(format!("mailto:{}", mat.as_str())),
            });
        }

        links
    }
}
```

**3.3.2 Terminal Integration** (2 days)
```rust
// libvt/src/terminal.rs
impl Terminal {
    pub fn register_link_provider(&mut self, provider: Box<dyn LinkProvider>) {
        self.link_provider = Some(provider);
    }

    pub fn get_links_for_row(&self, row: u16) -> Vec<Link> {
        if let Some(ref provider) = self.link_provider {
            if let Some(line) = self.buffer().get_line(row as usize) {
                let text = line.translate_to_string(true, None, None);
                return provider.provide_links(&text, row);
            }
        }
        Vec::new()
    }

    pub fn get_link_at_position(&self, col: u16, row: u16) -> Option<Link> {
        let links = self.get_links_for_row(row);
        links.into_iter().find(|link| {
            col >= link.start_col && col < link.end_col
        })
    }
}
```

**Deliverables**:
- ✅ LinkProvider trait
- ✅ Default URL/email detection
- ✅ registerLinkProvider() API
- ✅ get_links_for_row() method
- ✅ 20+ link detection tests

---

### 3.4 Character Joiner API (Week 21-22)

**Goal**: Custom character combining for ligatures

#### Tasks

**3.4.1 Character Joiner** (3 days)
```rust
// libvt/src/joiners.rs - NEW FILE
pub type CharacterJoiner = Box<dyn Fn(&str) -> Vec<(usize, usize)> + Send>;

pub struct CharacterJoinerRegistry {
    joiners: Vec<(usize, CharacterJoiner)>,
    next_id: usize,
}

impl CharacterJoinerRegistry {
    pub fn new() -> Self {
        Self {
            joiners: Vec::new(),
            next_id: 1,
        }
    }

    pub fn register(&mut self, joiner: CharacterJoiner) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.joiners.push((id, joiner));
        id
    }

    pub fn deregister(&mut self, id: usize) {
        self.joiners.retain(|(joiner_id, _)| *joiner_id != id);
    }

    pub fn apply_joiners(&self, text: &str) -> Vec<(usize, usize)> {
        let mut ranges = Vec::new();

        for (_, joiner) in &self.joiners {
            ranges.extend(joiner(text));
        }

        // Merge overlapping ranges
        ranges.sort_by_key(|r| r.0);
        // TODO: merge logic

        ranges
    }
}
```

**3.4.2 Terminal API** (1 day)
```rust
impl Terminal {
    pub fn register_character_joiner<F>(&mut self, joiner: F) -> usize
    where
        F: Fn(&str) -> Vec<(usize, usize)> + Send + 'static,
    {
        self.character_joiners.register(Box::new(joiner))
    }

    pub fn deregister_character_joiner(&mut self, id: usize) {
        self.character_joiners.deregister(id);
    }
}
```

**Deliverables**:
- ✅ CharacterJoiner system
- ✅ register/deregister API
- ✅ Range merging logic
- ✅ 10+ joiner tests

---

## Phase 3 Summary

**Deliverables**:
- ✅ Unicode handling (versions, graphemes, wide chars)
- ✅ Complete modes API (11 modes)
- ✅ Link provider system
- ✅ Character joiner system

**Testing**:
- ✅ 70+ advanced feature tests
- ✅ Unicode edge cases
- ✅ Link detection accuracy

**Code**: +1,500 LOC (total: ~7,500 LOC)

---

## Phase 4: NAPI-RS Bindings
**Duration**: 4-6 weeks
**Priority**: 🟡 HIGH
**Dependencies**: Phase 1-3

### 4.1 NAPI-RS Project Setup (Week 23)

**Goal**: Create libvt-napi project with basic TypeScript bindings

#### Tasks

**4.1.1 Project Structure** (1 day)
```bash
# Create new crate
cd wezterm
cargo new --lib libvt-napi

# Add to workspace
# Cargo.toml (root)
[workspace]
members = [
    # ... existing
    "libvt-napi",
]
```

```toml
# libvt-napi/Cargo.toml
[package]
name = "libvt-napi"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
libvt = { path = "../libvt" }
napi = "2.16"
napi-derive = "2.16"

[build-dependencies]
napi-build = "2.1"

[profile.release]
lto = true
strip = true
```

```json
// libvt-napi/package.json
{
  "name": "libvt-napi",
  "version": "0.1.0",
  "main": "index.js",
  "types": "index.d.ts",
  "napi": {
    "name": "libvt",
    "triples": {
      "defaults": true,
      "additional": [
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "x86_64-pc-windows-msvc",
        "x86_64-unknown-linux-gnu"
      ]
    }
  },
  "scripts": {
    "build": "napi build --platform --release",
    "build:debug": "napi build --platform",
    "prepublishOnly": "napi prepublish -t npm",
    "test": "node test.js"
  },
  "devDependencies": {
    "@napi-rs/cli": "^2.18.0"
  }
}
```

**4.1.2 Basic Terminal Wrapper** (3 days)
```rust
// libvt-napi/src/lib.rs
use napi::{bindgen_prelude::*, JsFunction};
use napi_derive::napi;
use libvt::{
    Terminal as LibVtTerminal,
    TerminalConfig,
    TerminalEvent,
    KeyCode,
    KeyModifiers,
};

#[napi(object)]
pub struct TerminalOptions {
    pub cols: u32,
    pub rows: u32,
    pub scrollback: Option<u32>,
    pub cursor_blink: Option<bool>,
    pub cursor_shape: Option<String>,
}

impl From<TerminalOptions> for TerminalConfig {
    fn from(opts: TerminalOptions) -> Self {
        let mut config = TerminalConfig::default();
        config.scrollback_lines = opts.scrollback.unwrap_or(10000) as usize;
        config.cursor_blink = opts.cursor_blink.unwrap_or(true);
        // ... more mappings
        config
    }
}

#[napi]
pub struct Terminal {
    inner: LibVtTerminal,

    // Event callbacks
    on_data: Option<ThreadsafeFunction<Vec<u8>>>,
    on_binary: Option<ThreadsafeFunction<Vec<u8>>>,
    on_bell: Option<ThreadsafeFunction<()>>,
    on_title_change: Option<ThreadsafeFunction<String>>,
    on_resize: Option<ThreadsafeFunction<ResizeEvent>>,
}

#[napi(object)]
pub struct ResizeEvent {
    pub cols: u32,
    pub rows: u32,
}

#[napi]
impl Terminal {
    #[napi(constructor)]
    pub fn new(options: Option<TerminalOptions>) -> Result<Self> {
        let opts = options.unwrap_or(TerminalOptions {
            cols: 80,
            rows: 24,
            scrollback: None,
            cursor_blink: None,
            cursor_shape: None,
        });

        let config: TerminalConfig = opts.into();
        let inner = LibVtTerminal::new(
            opts.cols as u16,
            opts.rows as u16,
            config
        );

        Ok(Terminal {
            inner,
            on_data: None,
            on_binary: None,
            on_bell: None,
            on_title_change: None,
            on_resize: None,
        })
    }

    #[napi]
    pub fn write(&mut self, env: Env, data: Either<String, Buffer>) -> Result<()> {
        let bytes = match data {
            Either::A(s) => s.as_bytes().to_vec(),
            Either::B(b) => b.to_vec(),
        };

        self.inner.write(&bytes);

        // Process events
        self.process_events(env)?;

        Ok(())
    }

    #[napi]
    pub fn writeln(&mut self, env: Env, data: Either<String, Buffer>) -> Result<()> {
        self.write(env, data)?;
        self.write(env, Either::A("\r\n".to_string()))?;
        Ok(())
    }

    #[napi(getter)]
    pub fn rows(&self) -> u32 {
        self.inner.size().1 as u32
    }

    #[napi(getter)]
    pub fn cols(&self) -> u32 {
        self.inner.size().0 as u32
    }

    #[napi]
    pub fn resize(&mut self, env: Env, cols: u32, rows: u32) -> Result<()> {
        self.inner.resize(cols as u16, rows as u16);

        // Trigger callback
        if let Some(ref tsfn) = self.on_resize {
            tsfn.call(
                Ok(ResizeEvent { cols, rows }),
                ThreadsafeFunctionCallMode::NonBlocking
            );
        }

        Ok(())
    }

    // Event handlers
    #[napi(ts_args_type = "callback: (data: Buffer) => void")]
    pub fn on_data(&mut self, env: Env, callback: JsFunction) -> Result<()> {
        let tsfn: ThreadsafeFunction<Vec<u8>> = callback
            .create_threadsafe_function(0, |ctx| {
                let buf = ctx.env.create_buffer_with_data(ctx.value)?;
                Ok(vec![buf.into_unknown()])
            })?;

        self.on_data = Some(tsfn);
        Ok(())
    }

    #[napi(ts_args_type = "callback: () => void")]
    pub fn on_bell(&mut self, env: Env, callback: JsFunction) -> Result<()> {
        let tsfn: ThreadsafeFunction<()> = callback
            .create_threadsafe_function(0, |ctx| Ok(vec![]))?;

        self.on_bell = Some(tsfn);
        Ok(())
    }

    // Helper to process events and trigger callbacks
    fn process_events(&mut self, env: Env) -> Result<()> {
        let events = self.inner.take_events();

        for event in events {
            match event {
                TerminalEvent::Bell => {
                    if let Some(ref tsfn) = self.on_bell {
                        tsfn.call(Ok(()), ThreadsafeFunctionCallMode::NonBlocking);
                    }
                }
                TerminalEvent::Data(data) => {
                    if let Some(ref tsfn) = self.on_data {
                        tsfn.call(Ok(data), ThreadsafeFunctionCallMode::NonBlocking);
                    }
                }
                TerminalEvent::TitleChanged(title) => {
                    if let Some(ref tsfn) = self.on_title_change {
                        tsfn.call(Ok(title), ThreadsafeFunctionCallMode::NonBlocking);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
```

**4.1.3 TypeScript Definitions** (1 day)
```typescript
// libvt-napi/index.d.ts
export interface TerminalOptions {
  cols?: number;
  rows?: number;
  scrollback?: number;
  cursorBlink?: boolean;
  cursorShape?: 'block' | 'underline' | 'bar';
}

export interface ResizeEvent {
  cols: number;
  rows: number;
}

export class Terminal {
  constructor(options?: TerminalOptions);

  // Properties
  readonly rows: number;
  readonly cols: number;

  // I/O
  write(data: string | Buffer): void;
  writeln(data: string | Buffer): void;

  // Sizing
  resize(cols: number, rows: number): void;

  // Events
  onData(callback: (data: Buffer) => void): void;
  onBinary(callback: (data: Buffer) => void): void;
  onBell(callback: () => void): void;
  onTitleChange(callback: (title: string) => void): void;
  onResize(callback: (event: ResizeEvent) => void): void;
}
```

**Deliverables**:
- ✅ NAPI-RS project structure
- ✅ Basic Terminal class
- ✅ Event callback infrastructure
- ✅ TypeScript definitions
- ✅ Build system

---

### 4.2 Buffer Access Bindings (Week 24-25)

**Goal**: Expose buffer API to TypeScript

#### Tasks

**4.2.1 Buffer Classes** (4 days)
```rust
// libvt-napi/src/buffer.rs
use napi::bindgen_prelude::*;
use napi_derive::napi;
use libvt::{Buffer as LibVtBuffer, BufferType};

#[napi]
pub struct Buffer {
    terminal_ref: External<LibVtTerminal>,
    buffer_type: BufferType,
}

#[napi]
impl Buffer {
    #[napi(getter)]
    pub fn cursor_x(&self, env: Env) -> Result<u32> {
        let term = unsafe { &*self.terminal_ref.as_ptr() };
        Ok(term.cursor_position().0 as u32)
    }

    #[napi(getter)]
    pub fn cursor_y(&self, env: Env) -> Result<u32> {
        let term = unsafe { &*self.terminal_ref.as_ptr() };
        Ok(term.cursor_position().1 as u32)
    }

    #[napi(getter)]
    pub fn viewport_y(&self, env: Env) -> Result<u32> {
        let term = unsafe { &*self.terminal_ref.as_ptr() };
        Ok(term.scroll_offset() as u32)
    }

    #[napi(getter)]
    pub fn base_y(&self, env: Env) -> Result<u32> {
        let term = unsafe { &*self.terminal_ref.as_ptr() };
        match self.buffer_type {
            BufferType::Normal => Ok(term.scrollback().len() as u32),
            BufferType::Alternate => Ok(0),
        }
    }

    #[napi(getter)]
    pub fn length(&self, env: Env) -> Result<u32> {
        let term = unsafe { &*self.terminal_ref.as_ptr() };
        match self.buffer_type {
            BufferType::Normal => {
                Ok((term.scrollback().len() + term.size().1 as usize) as u32)
            }
            BufferType::Alternate => Ok(term.size().1 as u32),
        }
    }

    #[napi]
    pub fn get_line(&self, env: Env, y: u32) -> Result<Option<BufferLine>> {
        let term = unsafe { &*self.terminal_ref.as_ptr() };
        let buffer = term.buffer();

        Ok(buffer.get_line(y as usize).map(|line| BufferLine {
            line_data: line,
            row: y,
        }))
    }
}

#[napi]
pub struct BufferLine {
    line_data: libvt::BufferLine,
    row: u32,
}

#[napi]
impl BufferLine {
    #[napi(getter)]
    pub fn is_wrapped(&self) -> bool {
        self.line_data.is_wrapped()
    }

    #[napi(getter)]
    pub fn length(&self) -> u32 {
        self.line_data.length() as u32
    }

    #[napi]
    pub fn translate_to_string(
        &self,
        trim_right: Option<bool>,
        start_column: Option<u32>,
        end_column: Option<u32>,
    ) -> String {
        self.line_data.translate_to_string(
            trim_right.unwrap_or(false),
            start_column.map(|c| c as usize),
            end_column.map(|c| c as usize),
        )
    }

    #[napi]
    pub fn get_cell(&self, x: u32) -> Option<BufferCell> {
        self.line_data.get_cell(x as usize).map(|cell| BufferCell {
            cell_data: cell,
        })
    }
}

#[napi]
pub struct BufferCell {
    cell_data: libvt::BufferCell,
}

#[napi]
impl BufferCell {
    #[napi]
    pub fn get_chars(&self) -> String {
        self.cell_data.get_chars()
    }

    #[napi]
    pub fn get_width(&self) -> u32 {
        self.cell_data.get_width() as u32
    }

    #[napi]
    pub fn is_bold(&self) -> bool {
        self.cell_data.is_bold()
    }

    #[napi]
    pub fn is_italic(&self) -> bool {
        self.cell_data.is_italic()
    }

    // ... all other cell methods
}
```

**4.2.2 Terminal Buffer Property** (1 day)
```rust
// libvt-napi/src/lib.rs
#[napi]
impl Terminal {
    #[napi(getter)]
    pub fn buffer(&self, env: Env) -> Result<Buffer> {
        let ptr = &self.inner as *const LibVtTerminal;
        Ok(Buffer {
            terminal_ref: env.create_external(ptr, None)?,
            buffer_type: self.inner.buffers.active,
        })
    }
}
```

**Deliverables**:
- ✅ Buffer, BufferLine, BufferCell classes
- ✅ All buffer API methods
- ✅ TypeScript definitions
- ✅ 20+ integration tests

---

### 4.3 Complete API Bindings (Week 26-27)

**Goal**: All remaining xterm.js-compatible methods

#### Tasks

**4.3.1 Selection Methods** (2 days)
```rust
#[napi]
impl Terminal {
    #[napi]
    pub fn select(&mut self, col: u32, row: u32, length: u32) {
        self.inner.select(col as u16, row as u16, length as u16);
    }

    #[napi]
    pub fn select_all(&mut self) {
        self.inner.select_all();
    }

    #[napi]
    pub fn select_lines(&mut self, start: u32, end: u32) {
        self.inner.select_lines(start as u16, end as u16);
    }

    #[napi]
    pub fn has_selection(&self) -> bool {
        self.inner.has_selection()
    }

    #[napi]
    pub fn get_selection(&self) -> Option<String> {
        self.inner.selected_text()
    }

    #[napi]
    pub fn clear_selection(&mut self) {
        self.inner.clear_selection();
    }
}
```

**4.3.2 Scrolling Methods** (1 day)
```rust
#[napi]
impl Terminal {
    #[napi]
    pub fn scroll_lines(&mut self, amount: i32) {
        self.inner.scroll_lines(amount as isize);
    }

    #[napi]
    pub fn scroll_pages(&mut self, page_count: i32) {
        self.inner.scroll_pages(page_count as isize);
    }

    #[napi]
    pub fn scroll_to_top(&mut self) {
        self.inner.scroll_to_top();
    }

    #[napi]
    pub fn scroll_to_bottom(&mut self) {
        self.inner.scroll_to_bottom();
    }

    #[napi]
    pub fn scroll_to_line(&mut self, line: u32) {
        self.inner.scroll_to_line(line as usize);
    }
}
```

**4.3.3 Input Methods** (1 day)
```rust
#[napi]
impl Terminal {
    #[napi]
    pub fn paste(&mut self, env: Env, data: String) -> Result<()> {
        self.inner.paste(&data);
        self.process_events(env)?;
        Ok(())
    }

    #[napi]
    pub fn input(&mut self, env: Env, data: String, was_user_input: Option<bool>) -> Result<()> {
        self.inner.input(&data, was_user_input.unwrap_or(true));
        self.process_events(env)?;
        Ok(())
    }
}
```

**4.3.4 Marker Methods** (1 day)
```rust
#[napi]
impl Terminal {
    #[napi]
    pub fn register_marker(&mut self, cursor_y_offset: Option<i32>) -> u32 {
        let id = self.inner.register_marker(cursor_y_offset);
        id.0 as u32 // Convert MarkerId to u32
    }

    #[napi(getter)]
    pub fn markers(&self) -> Vec<u32> {
        self.inner.markers()
            .into_iter()
            .map(|m| m.id().0 as u32)
            .collect()
    }
}
```

**Deliverables**:
- ✅ All selection methods (6 methods)
- ✅ All scrolling methods (5 methods)
- ✅ Input methods (2 methods)
- ✅ Marker methods (2 methods)
- ✅ Complete TypeScript definitions

---

### 4.4 Testing & Documentation (Week 28)

**Goal**: Integration tests and API documentation

#### Tasks

**4.4.1 Node.js Tests** (3 days)
```javascript
// libvt-napi/test.js
const { Terminal } = require('./index.js');

// Basic functionality
function test_basic() {
    const term = new Terminal({ cols: 80, rows: 24 });

    console.assert(term.rows === 24, 'rows should be 24');
    console.assert(term.cols === 80, 'cols should be 80');

    term.write('Hello World');
    // Add assertions for buffer content

    console.log('✓ Basic test passed');
}

// Event handling
function test_events() {
    const term = new Terminal({ cols: 80, rows: 24 });

    let bellFired = false;
    term.onBell(() => {
        bellFired = true;
    });

    term.write('\x07'); // Bell

    setTimeout(() => {
        console.assert(bellFired, 'Bell event should fire');
        console.log('✓ Event test passed');
    }, 100);
}

// Selection
function test_selection() {
    const term = new Terminal({ cols: 80, rows: 24 });

    term.write('Hello World');
    term.selectAll();

    console.assert(term.hasSelection(), 'Should have selection');
    const text = term.getSelection();
    console.assert(text.includes('Hello'), 'Selection should include text');

    console.log('✓ Selection test passed');
}

// Run all tests
test_basic();
test_events();
test_selection();
```

**4.4.2 Documentation** (2 days)
```markdown
# libvt-napi

NAPI-RS bindings for libvt terminal emulator.

## Installation

```bash
npm install libvt-napi
```

## Usage

```javascript
const { Terminal } = require('libvt-napi');

const term = new Terminal({ cols: 80, rows: 24 });

// Write data
term.write('Hello \x1b[1;32mWorld\x1b[0m\n');

// Handle events
term.onData((data) => {
    console.log('User input:', data);
});

term.onBell(() => {
    console.log('Bell!');
});

// Selection
term.selectAll();
const text = term.getSelection();
```

## API

See [API.md](./API.md) for complete API documentation.
```

**Deliverables**:
- ✅ 50+ Node.js integration tests
- ✅ API documentation
- ✅ Usage examples
- ✅ Performance benchmarks

---

## Phase 4 Summary

**Deliverables**:
- ✅ Complete NAPI-RS bindings (30+ methods)
- ✅ TypeScript definitions
- ✅ Event callback system
- ✅ Buffer access API
- ✅ Integration tests

**Testing**:
- ✅ 50+ Node.js tests
- ✅ API compatibility verification

**Code**: +2,000 LOC (total: ~9,500 LOC)

---

## Phase 5: VSCode Integration
**Duration**: 4-6 weeks
**Priority**: 🟢 INTEGRATION
**Dependencies**: Phase 1-4

### 5.1 Canvas Renderer (Week 29-30)

**Goal**: TypeScript Canvas/WebGL renderer for terminal cells

#### Tasks

**5.1.1 Renderer Architecture** (3 days)
```typescript
// libvt-renderer/src/renderer.ts
import { Terminal } from 'libvt-napi';

export interface RendererOptions {
    canvas: HTMLCanvasElement;
    fontSize: number;
    fontFamily: string;
    lineHeight: number;
    devicePixelRatio?: number;
}

export class TerminalRenderer {
    private canvas: HTMLCanvasElement;
    private ctx: CanvasRenderingContext2D;
    private terminal: Terminal;

    private fontSize: number;
    private fontFamily: string;
    private lineHeight: number;

    private charWidth: number;
    private charHeight: number;

    private glyphAtlas: GlyphAtlas;

    constructor(terminal: Terminal, options: RendererOptions) {
        this.terminal = terminal;
        this.canvas = options.canvas;
        this.fontSize = options.fontSize;
        this.fontFamily = options.fontFamily;
        this.lineHeight = options.lineHeight;

        const ctx = this.canvas.getContext('2d', {
            alpha: false,
            desynchronized: true,
        });
        if (!ctx) throw new Error('Failed to get 2D context');
        this.ctx = ctx;

        // Calculate metrics
        this.measureFont();

        // Create glyph atlas
        this.glyphAtlas = new GlyphAtlas(this.ctx, this.fontSize, this.fontFamily);

        // Set canvas size
        this.resize();
    }

    private measureFont(): void {
        this.ctx.font = `${this.fontSize}px ${this.fontFamily}`;
        const metrics = this.ctx.measureText('W');
        this.charWidth = metrics.width;
        this.charHeight = this.fontSize * this.lineHeight;
    }

    public resize(): void {
        const cols = this.terminal.cols;
        const rows = this.terminal.rows;

        this.canvas.width = cols * this.charWidth;
        this.canvas.height = rows * this.charHeight;
    }

    public render(): void {
        // Clear canvas
        this.ctx.fillStyle = '#000000';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

        const buffer = this.terminal.buffer;

        // Render each visible line
        for (let row = 0; row < this.terminal.rows; row++) {
            const line = buffer.getLine(row);
            if (!line) continue;

            this.renderLine(line, row);
        }

        // Render cursor
        this.renderCursor();
    }

    private renderLine(line: any, row: number): void {
        const y = row * this.charHeight;

        for (let col = 0; col < line.length; col++) {
            const cell = line.getCell(col);
            if (!cell) continue;

            const x = col * this.charWidth;

            // Render background
            if (cell.getBgColorMode() !== 0) {
                this.ctx.fillStyle = this.getColorString(
                    cell.getBgColorMode(),
                    cell.getBgColor()
                );
                this.ctx.fillRect(x, y, this.charWidth, this.charHeight);
            }

            // Render character
            const char = cell.getChars();
            if (char && char !== ' ') {
                this.ctx.fillStyle = this.getColorString(
                    cell.getFgColorMode(),
                    cell.getFgColor()
                );

                const glyph = this.glyphAtlas.getGlyph(char);
                if (glyph) {
                    this.ctx.drawImage(
                        glyph.canvas,
                        x, y + this.fontSize
                    );
                }
            }
        }
    }

    private renderCursor(): void {
        // TODO: Implement cursor rendering
    }

    private getColorString(mode: number, color: number): string {
        // Convert color to CSS string
        // mode: 0=default, 1=palette, 2=rgb
        if (mode === 2) {
            const r = (color >> 16) & 0xFF;
            const g = (color >> 8) & 0xFF;
            const b = color & 0xFF;
            return `rgb(${r}, ${g}, ${b})`;
        } else if (mode === 1) {
            // Palette color
            return this.getPaletteColor(color);
        }
        return '#FFFFFF'; // Default
    }

    private getPaletteColor(index: number): string {
        // ANSI color palette
        const palette = [
            '#000000', '#800000', '#008000', '#808000',
            '#000080', '#800080', '#008080', '#c0c0c0',
            '#808080', '#ff0000', '#00ff00', '#ffff00',
            '#0000ff', '#ff00ff', '#00ffff', '#ffffff',
            // ... 256 colors
        ];
        return palette[index] || '#FFFFFF';
    }
}
```

**5.1.2 Glyph Atlas** (2 days)
```typescript
// libvt-renderer/src/atlas.ts
export class GlyphAtlas {
    private cache: Map<string, GlyphInfo>;
    private ctx: CanvasRenderingContext2D;
    private fontSize: number;
    private fontFamily: string;

    constructor(ctx: CanvasRenderingContext2D, fontSize: number, fontFamily: string) {
        this.ctx = ctx;
        this.fontSize = fontSize;
        this.fontFamily = fontFamily;
        this.cache = new Map();
    }

    public getGlyph(char: string): GlyphInfo | null {
        if (this.cache.has(char)) {
            return this.cache.get(char)!;
        }

        // Render glyph to canvas
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d')!;

        ctx.font = `${this.fontSize}px ${this.fontFamily}`;
        const metrics = ctx.measureText(char);

        canvas.width = Math.ceil(metrics.width);
        canvas.height = this.fontSize;

        ctx.font = `${this.fontSize}px ${this.fontFamily}`;
        ctx.fillStyle = '#FFFFFF';
        ctx.fillText(char, 0, this.fontSize);

        const glyph: GlyphInfo = {
            char,
            canvas,
            width: metrics.width,
            height: this.fontSize,
        };

        this.cache.set(char, glyph);
        return glyph;
    }
}

interface GlyphInfo {
    char: string;
    canvas: HTMLCanvasElement;
    width: number;
    height: number;
}
```

**Deliverables**:
- ✅ Canvas renderer architecture
- ✅ Glyph atlas caching
- ✅ Basic rendering (text, colors)
- ✅ Cursor rendering

---

### 5.2 Input Handling (Week 31-32)

**Goal**: Keyboard, mouse, IME input

#### Tasks

**5.2.1 Keyboard Input** (3 days)
```typescript
// libvt-renderer/src/input.ts
import { Terminal } from 'libvt-napi';

export class InputHandler {
    private terminal: Terminal;
    private textarea: HTMLTextAreaElement;

    constructor(terminal: Terminal, container: HTMLElement) {
        this.terminal = terminal;

        // Create invisible textarea for composition
        this.textarea = document.createElement('textarea');
        this.textarea.style.position = 'absolute';
        this.textarea.style.opacity = '0';
        this.textarea.style.pointerEvents = 'none';
        container.appendChild(this.textarea);

        this.attachKeyboardListeners();
    }

    private attachKeyboardListeners(): void {
        this.textarea.addEventListener('keydown', (e) => {
            this.handleKeyDown(e);
        });

        this.textarea.addEventListener('input', (e) => {
            this.handleInput(e);
        });

        this.textarea.addEventListener('paste', (e) => {
            this.handlePaste(e);
        });
    }

    private handleKeyDown(e: KeyboardEvent): void {
        const key = this.translateKey(e);
        if (!key) return;

        // Let terminal encode the key
        // This will trigger onData event

        e.preventDefault();
    }

    private handleInput(e: Event): void {
        const target = e.target as HTMLTextAreaElement;
        const data = target.value;

        if (data) {
            this.terminal.input(data, true);
            target.value = '';
        }
    }

    private handlePaste(e: ClipboardEvent): void {
        e.preventDefault();
        const data = e.clipboardData?.getData('text');
        if (data) {
            this.terminal.paste(data);
        }
    }

    private translateKey(e: KeyboardEvent): string | null {
        // Map DOM KeyboardEvent to terminal key codes
        // This is simplified - real implementation needs full mapping

        if (e.key === 'Enter') return '\r';
        if (e.key === 'Tab') return '\t';
        if (e.key === 'Backspace') return '\x7F';
        if (e.key === 'Escape') return '\x1b';

        if (e.key === 'ArrowUp') return '\x1b[A';
        if (e.key === 'ArrowDown') return '\x1b[B';
        if (e.key === 'ArrowRight') return '\x1b[C';
        if (e.key === 'ArrowLeft') return '\x1b[D';

        if (e.ctrlKey && e.key.length === 1) {
            const code = e.key.toUpperCase().charCodeAt(0) - 64;
            return String.fromCharCode(code);
        }

        return null;
    }
}
```

**5.2.2 Mouse Input** (2 days)
```typescript
export class MouseHandler {
    private terminal: Terminal;
    private canvas: HTMLCanvasElement;
    private charWidth: number;
    private charHeight: number;

    constructor(terminal: Terminal, canvas: HTMLCanvasElement, charWidth: number, charHeight: number) {
        this.terminal = terminal;
        this.canvas = canvas;
        this.charWidth = charWidth;
        this.charHeight = charHeight;

        this.attachMouseListeners();
    }

    private attachMouseListeners(): void {
        this.canvas.addEventListener('mousedown', (e) => {
            this.handleMouseDown(e);
        });

        this.canvas.addEventListener('mousemove', (e) => {
            this.handleMouseMove(e);
        });

        this.canvas.addEventListener('mouseup', (e) => {
            this.handleMouseUp(e);
        });

        this.canvas.addEventListener('wheel', (e) => {
            this.handleWheel(e);
        });
    }

    private getCellPosition(e: MouseEvent): { col: number, row: number } {
        const rect = this.canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        return {
            col: Math.floor(x / this.charWidth),
            row: Math.floor(y / this.charHeight),
        };
    }

    private handleMouseDown(e: MouseEvent): void {
        const pos = this.getCellPosition(e);

        // Start selection
        this.terminal.select(pos.col, pos.row, 1);
    }

    private handleMouseMove(e: MouseEvent): void {
        if (e.buttons === 1) {
            // Extend selection
            const pos = this.getCellPosition(e);
            // TODO: Extend selection to current position
        }
    }

    private handleMouseUp(e: MouseEvent): void {
        // Finalize selection
    }

    private handleWheel(e: WheelEvent): void {
        e.preventDefault();
        const delta = e.deltaY > 0 ? 1 : -1;
        this.terminal.scrollLines(delta * 3);
    }
}
```

**Deliverables**:
- ✅ Keyboard input handler
- ✅ Mouse input handler
- ✅ IME support (composition)
- ✅ Paste handling

---

### 5.3 VSCode Extension (Week 33-34)

**Goal**: VSCode terminal provider using libvt

#### Tasks

**5.3.1 Extension Structure** (2 days)
```typescript
// vscode-libvt/src/extension.ts
import * as vscode from 'vscode';
import { LibVtTerminal } from './terminal';

export function activate(context: vscode.ExtensionContext) {
    context.subscriptions.push(
        vscode.window.registerTerminalProfileProvider('libvt.terminal', {
            provideTerminalProfile(): vscode.TerminalProfile {
                return new vscode.TerminalProfile({
                    name: 'LibVT Terminal',
                    shellPath: '/bin/bash', // or detect
                });
            },
        })
    );

    // Register custom terminal
    context.subscriptions.push(
        vscode.commands.registerCommand('libvt.createTerminal', () => {
            const terminal = new LibVtTerminal();
            terminal.show();
        })
    );
}
```

**5.3.2 Terminal Provider** (4 days)
```typescript
// vscode-libvt/src/terminal.ts
import * as vscode from 'vscode';
import { Terminal } from 'libvt-napi';
import { TerminalRenderer } from 'libvt-renderer';
import * as pty from 'node-pty';

export class LibVtTerminal implements vscode.Pseudoterminal {
    private writeEmitter = new vscode.EventEmitter<string>();
    private closeEmitter = new vscode.EventEmitter<void>();

    onDidWrite: vscode.Event<string> = this.writeEmitter.event;
    onDidClose: vscode.Event<void> = this.closeEmitter.event;

    private terminal: Terminal;
    private ptyProcess: pty.IPty;

    constructor() {
        // Create libvt terminal
        this.terminal = new Terminal({
            cols: 80,
            rows: 24,
        });

        // Set up event handlers
        this.terminal.onData((data) => {
            // Send to PTY
            this.ptyProcess.write(data.toString());
        });

        // Create PTY
        this.ptyProcess = pty.spawn(
            process.env.SHELL || '/bin/bash',
            [],
            {
                cols: 80,
                rows: 24,
                cwd: process.cwd(),
                env: process.env as any,
            }
        );

        // PTY data -> terminal
        this.ptyProcess.onData((data) => {
            this.terminal.write(data);
            this.writeEmitter.fire(data);
        });

        this.ptyProcess.onExit(() => {
            this.closeEmitter.fire();
        });
    }

    open(initialDimensions: vscode.TerminalDimensions | undefined): void {
        if (initialDimensions) {
            this.setDimensions({
                columns: initialDimensions.columns,
                rows: initialDimensions.rows,
            });
        }
    }

    close(): void {
        this.ptyProcess.kill();
    }

    handleInput(data: string): void {
        this.terminal.input(data, true);
    }

    setDimensions(dimensions: vscode.TerminalDimensions): void {
        this.terminal.resize(dimensions.columns, dimensions.rows);
        this.ptyProcess.resize(dimensions.columns, dimensions.rows);
    }

    public show(): void {
        const term = vscode.window.createTerminal({
            name: 'LibVT',
            pty: this,
        });
        term.show();
    }
}
```

**5.3.3 Configuration** (1 day)
```json
// vscode-libvt/package.json
{
  "name": "vscode-libvt",
  "displayName": "LibVT Terminal",
  "description": "High-performance terminal using libvt",
  "version": "0.1.0",
  "engines": {
    "vscode": "^1.80.0"
  },
  "categories": ["Other"],
  "activationEvents": ["onStartupFinished"],
  "main": "./out/extension.js",
  "contributes": {
    "commands": [
      {
        "command": "libvt.createTerminal",
        "title": "Create LibVT Terminal"
      }
    ],
    "terminal": {
      "profiles": [
        {
          "id": "libvt.terminal",
          "title": "LibVT",
          "icon": "terminal"
        }
      ]
    }
  },
  "dependencies": {
    "libvt-napi": "^0.1.0",
    "libvt-renderer": "^0.1.0",
    "node-pty": "^1.0.0"
  }
}
```

**Deliverables**:
- ✅ VSCode extension structure
- ✅ Terminal provider implementation
- ✅ PTY integration
- ✅ Configuration schema
- ✅ Theme integration

---

## Phase 5 Summary

**Deliverables**:
- ✅ Canvas/WebGL renderer (TypeScript)
- ✅ Input handling (keyboard, mouse, IME)
- ✅ VSCode extension
- ✅ PTY integration
- ✅ End-to-end working terminal

**Testing**:
- ✅ Rendering tests
- ✅ Input tests
- ✅ VSCode integration tests

**Code**: +2,500 LOC TypeScript (total: ~9,500 LOC Rust + 2,500 LOC TS)

---

## Overall Project Summary

### Timeline

| Phase | Duration | Cumulative | Milestone |
|-------|----------|------------|-----------|
| **Phase 1: Core API** | 8-10 weeks | 10 weeks | Critical xterm.js compatibility |
| **Phase 2: Buffer Access** | 3-4 weeks | 14 weeks | Complete buffer API |
| **Phase 3: Advanced** | 6-8 weeks | 22 weeks | Unicode, modes, links |
| **Phase 4: NAPI-RS** | 4-6 weeks | 28 weeks | TypeScript bindings |
| **Phase 5: VSCode** | 4-6 weeks | 34 weeks | Working integration |

**Total**: 25-34 weeks (6-8 months)

### Code Metrics

| Component | LOC | Tests |
|-----------|-----|-------|
| libvt (Rust core) | ~7,500 | 170+ |
| libvt-napi (bindings) | ~2,000 | 50+ |
| libvt-renderer (TS) | ~1,500 | 30+ |
| vscode-libvt (extension) | ~1,000 | 20+ |
| **Total** | **~12,000** | **270+** |

### Success Metrics

**Performance** (Target):
- ✅ Parse throughput: >100 MB/s (vs xterm.js 30-50 MB/s)
- ✅ Memory: <100 MB for 100k scrollback (vs xterm.js 100-200 MB)
- ✅ Latency: 0ms GC pauses (vs xterm.js ~10ms)
- ✅ FPS: Stable 60 FPS rendering

**Compatibility**:
- ✅ vim workflow (edit, save, quit)
- ✅ emacs workflow
- ✅ tmux (panes, status line)
- ✅ htop (colors, mouse)
- ✅ vttest compatibility (90%+)

**Quality**:
- ✅ 270+ tests passing
- ✅ Zero unsafe code
- ✅ Zero clippy warnings
- ✅ Full API documentation

---

## Risk Mitigation

### Technical Risks

**Risk 1**: Performance doesn't meet expectations
- **Mitigation**: Benchmark early (Phase 1), optimize incrementally
- **Fallback**: Focus on correctness, optimize later

**Risk 2**: NAPI-RS bridge overhead
- **Mitigation**: Use zero-copy where possible, minimize allocations
- **Fallback**: Investigate alternative FFI approaches

**Risk 3**: VSCode API changes
- **Mitigation**: Use stable VSCode APIs, version pinning
- **Fallback**: Provide standalone terminal app

**Risk 4**: Compatibility issues with edge cases
- **Mitigation**: Extensive testing with real applications
- **Fallback**: Document known limitations

### Schedule Risks

**Risk 1**: Feature scope creep
- **Mitigation**: Strict phase boundaries, MVP-first approach
- **Fallback**: De-scope Phase 3 features

**Risk 2**: Dependencies on external libraries
- **Mitigation**: Evaluate dependencies early, have alternatives
- **Fallback**: Implement critical features in-house

---

## Next Steps

### Immediate Actions (Week 1)

1. **Set up development environment**
   - Rust toolchain (latest stable)
   - Node.js 18+ for NAPI-RS
   - VSCode with Rust Analyzer

2. **Create project structure**
   - Initialize libvt-napi crate
   - Set up build system
   - Create test harness

3. **Start Phase 1.1** (Event System)
   - Expand TerminalEvent enum
   - Implement event emission
   - Write first tests

### Weekly Milestones

**Week 1-2**: Event system complete
**Week 3**: Alternate screen working (vim test passes)
**Week 4-5**: Selection complete
**Week 6**: Scrolling complete
**Week 10**: Phase 1 complete (MVP demo)
**Week 14**: Phase 2 complete (buffer access)
**Week 22**: Phase 3 complete (advanced features)
**Week 28**: Phase 4 complete (NAPI-RS bindings)
**Week 34**: Phase 5 complete (VSCode integration)

---

## Conclusion

This plan provides a complete roadmap for transforming libvt into a production-ready VSCode terminal backend. The phased approach ensures:

1. **Early value**: Phase 1-2 (14 weeks) provides MVP functionality
2. **Incremental testing**: Each phase has deliverables and tests
3. **Risk management**: Clear dependencies and fallback plans
4. **Quality focus**: 270+ tests, documentation, benchmarks

With focused execution, libvt can become a compelling high-performance alternative to xterm.js, bringing Rust's safety and performance benefits to VSCode's terminal.

---

**Document**: IMPLEMENTATION_PLAN.md
**Version**: 1.0
**Author**: Claude Code
**Date**: 2025-11-18
**Status**: Ready for Implementation
