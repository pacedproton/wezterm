# LibVT as xterm.js Replacement - Complete Analysis

## Executive Summary

This document analyzes **xterm.js** (VSCode's current terminal frontend) and details how **libvt** can be positioned as a high-performance Rust replacement. xterm.js is a TypeScript terminal emulator rendering to canvas/WebGL, while libvt would provide the terminal state machine with rendering delegated to the application.

**Key Insight**: xterm.js is a **complete terminal emulator + renderer**. libvt is **just the emulator core**. A libvt-based replacement would require:
1. **libvt** (Rust) - Terminal state machine & escape sequence parsing
2. **Renderer** (Canvas/WebGL in TypeScript or Rust→WASM)
3. **NAPI-RS bindings** - Connect Rust libvt to TypeScript frontend

---

## xterm.js Architecture

### Component Breakdown

```
┌──────────────────────────────────────────────────────┐
│              VSCode (Electron App)                    │
│  ┌────────────────────────────────────────────────┐  │
│  │         xterm.js (TypeScript)                   │  │
│  │  ┌──────────────────────────────────────────┐  │  │
│  │  │  Terminal Class (Public API)              │  │  │
│  │  │  - write(), writeln(), paste()            │  │  │
│  │  │  - Event emitters (onData, onBinary, etc) │  │  │
│  │  │  - Buffer access (buffer.active)          │  │  │
│  │  └────────────┬─────────────────────────────┘  │  │
│  │               │                                  │  │
│  │  ┌────────────▼─────────────────────────────┐  │  │
│  │  │  Core Services (DI Architecture)         │  │  │
│  │  │  - BufferService (screen state)          │  │  │
│  │  │  - InputHandler (escape sequences)       │  │  │
│  │  │  - Parser (vtparse-based)                │  │  │
│  │  │  - OptionsService (configuration)        │  │  │
│  │  │  - CoreService (cursor, modes)           │  │  │
│  │  └────────────┬─────────────────────────────┘  │  │
│  │               │                                  │  │
│  │  ┌────────────▼─────────────────────────────┐  │  │
│  │  │  Rendering Layers (Canvas/WebGL)         │  │  │
│  │  │  1. TextRenderLayer (fg/bg)              │  │  │
│  │  │  2. SelectionRenderLayer                 │  │  │
│  │  │  3. LinkRenderLayer (hyperlinks)         │  │  │
│  │  │  4. CursorRenderLayer (blinking)         │  │  │
│  │  └──────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────┘  │
│                         │                             │
│  ┌──────────────────────▼──────────────────────────┐ │
│  │         node-pty (Native Module)                 │ │
│  │  - Process spawning (bash, zsh, pwsh, etc.)     │ │
│  │  - PTY management (conpty, winpty, posix pty)   │ │
│  └──────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

### Rendering Strategy

xterm.js uses **layered canvas rendering** for performance:
1. **TextRenderLayer** - Background and foreground of content
2. **SelectionRenderLayer** - Selected region overlay
3. **LinkRenderLayer** - Hyperlink decoration
4. **CursorRenderLayer** - Blinking cursor

**Optimization**: Only repaint changed layers, not the whole canvas.

---

## xterm.js API Surface Analysis

### Core Terminal Class API

| Category | Methods/Properties | Count |
|----------|-------------------|-------|
| **Properties** | element, textarea, rows, cols, buffer, markers, parser, unicode, modes, options | 10 |
| **I/O Methods** | write, writeln, paste, input | 4 |
| **Display** | clear, reset, refresh, clearTextureAtlas | 4 |
| **Sizing** | resize, open | 2 |
| **Scrolling** | scrollLines, scrollPages, scrollToTop, scrollToBottom, scrollToLine | 5 |
| **Selection** | select, selectAll, selectLines, hasSelection, getSelection, getSelectionPosition, clearSelection | 7 |
| **Focus** | focus, blur | 2 |
| **Markers** | registerMarker, registerDecoration | 2 |
| **Handlers** | attachCustomKeyEventHandler, attachCustomWheelEventHandler, registerLinkProvider, registerCharacterJoiner, deregisterCharacterJoiner | 5 |
| **Lifecycle** | loadAddon, dispose | 2 |
| **Events** | onBell, onBinary, onCursorMove, onData, onKey, onLineFeed, onRender, onWriteParsed, onResize, onScroll, onSelectionChange, onTitleChange | 12 |
| **TOTAL** | | **55+** |

### Buffer API (`terminal.buffer`)

```typescript
interface IBufferNamespace {
    active: IBuffer;      // Normal or alternate buffer
    normal: IBuffer;      // Normal screen buffer
    alternate: IBuffer;   // Alternate screen buffer
    onBufferChange: IEvent<IBuffer>;
}

interface IBuffer {
    type: 'normal' | 'alternate';
    cursorX: number;
    cursorY: number;
    viewportY: number;      // Top of viewport (scrollback aware)
    baseY: number;          // Top of normal buffer
    length: number;         // Total lines (including scrollback)
    getLine(y: number): IBufferLine | undefined;
    getNullCell(): IBufferCell;
}

interface IBufferLine {
    isWrapped: boolean;
    length: number;
    getCell(x: number, cell?: IBufferCell): IBufferCell | undefined;
    translateToString(trimRight?: boolean, startColumn?: number, endColumn?: number): string;
}

interface IBufferCell {
    getChars(): string;
    getWidth(): number;
    getCode(): number;
    isBold(): number;
    isItalic(): number;
    isDim(): number;
    isUnderline(): number;
    isBlink(): number;
    isInverse(): number;
    isInvisible(): number;
    isStrikethrough(): number;
    getFgColorMode(): number;
    getBgColorMode(): number;
    getFgColor(): number;
    getBgColor(): number;
}
```

### Parser API (`terminal.parser`)

```typescript
interface IParser {
    registerCsiHandler(id: IFunctionIdentifier, callback: (params: IParams) => boolean | Promise<boolean>): IDisposable;
    addCsiHandler(id: IFunctionIdentifier, callback: (params: IParams) => boolean | Promise<boolean>): IDisposable;
    registerDcsHandler(id: IFunctionIdentifier, callback: (data: string, param: IParams) => boolean | Promise<boolean>): IDisposable;
    addDcsHandler(id: IFunctionIdentifier, callback: (data: string, param: IParams) => boolean | Promise<boolean>): IDisposable;
    registerEscHandler(id: IFunctionIdentifier, handler: () => boolean | Promise<boolean>): IDisposable;
    addEscHandler(id: IFunctionIdentifier, handler: () => boolean | Promise<boolean>): IDisposable;
    registerOscHandler(ident: number, callback: (data: string) => boolean | Promise<boolean>): IDisposable;
    addOscHandler(ident: number, callback: (data: string) => boolean | Promise<boolean>): IDisposable;
}
```

**Key Feature**: Applications can register custom handlers for escape sequences!

### Options API (`ITerminalOptions`)

**Rendering Options** (17):
- `fontSize`, `fontFamily`, `fontWeight`, `fontWeightBold`
- `lineHeight`, `letterSpacing`, `customGlyphs`
- `cursorBlink`, `cursorStyle`, `cursorWidth`, `cursorInactiveStyle`
- `theme` (colors)
- `minimumContrastRatio`, `drawBoldTextInBrightColors`
- `allowTransparency`, `rescaleOverlappingGlyphs`
- `reflowCursorLine`

**Behavior Options** (12):
- `convertEol`, `disableStdin`, `ignoreBracketedPasteMode`
- `macOptionIsMeta`, `macOptionClickForcesSelection`
- `altClickMovesCursor`, `rightClickSelectsWord`
- `scrollback`, `scrollOnUserInput`, `scrollOnEraseInDisplay`
- `wordSeparator`, `tabStopWidth`

**Platform Options** (2):
- `windowsMode`, `windowsPty`

**Advanced Options** (6):
- `allowProposedApi`, `linkHandler`, `documentOverride`
- `windowOptions`, `overviewRuler`, `screenReaderMode`

**Total**: ~40 configuration options

---

## Official xterm.js Addons

| Addon | Purpose | VSCode Usage | Priority |
|-------|---------|--------------|----------|
| **@xterm/addon-attach** | WebSocket connection to server process | ❌ No (uses IPC) | LOW |
| **@xterm/addon-canvas** | 2D canvas renderer (fallback) | ✅ Yes | HIGH |
| **@xterm/addon-clipboard** | System clipboard access | ✅ Yes | HIGH |
| **@xterm/addon-fit** | Auto-resize terminal to container | ✅ Yes | HIGH |
| **@xterm/addon-image** | Sixel + iTerm2 inline images | 🔜 Experimental | MEDIUM |
| **@xterm/addon-ligatures** | Programming font ligatures | ❌ No | LOW |
| **@xterm/addon-search** | In-terminal search | ✅ Yes | HIGH |
| **@xterm/addon-serialize** | Export terminal state to VT/HTML | ❌ No | LOW |
| **@xterm/addon-unicode-graphemes** | Grapheme cluster support | 🔜 Experimental | MEDIUM |
| **@xterm/addon-unicode11** | Unicode 11 character widths | ✅ Yes | MEDIUM |
| **@xterm/addon-web-links** | Auto-detect and click URLs | ✅ Yes | HIGH |
| **@xterm/addon-webgl** | WebGL2 renderer (high performance) | ✅ Yes | HIGH |

**VSCode primarily uses**: canvas/webgl, clipboard, fit, search, web-links, unicode11

---

## libvt → xterm.js API Mapping

### ✅ Already Implemented in libvt

| xterm.js API | libvt Equivalent | Status | Notes |
|--------------|------------------|--------|-------|
| `write(data)` | `terminal.write(data)` | ✅ Complete | Both support bytes/strings |
| `rows` | `terminal.size().1` | ✅ Complete | Returns (cols, rows) |
| `cols` | `terminal.size().0` | ✅ Complete | |
| `buffer.cursorX` | `terminal.cursor_position().0` | ✅ Complete | |
| `buffer.cursorY` | `terminal.cursor_position().1` | ✅ Complete | |
| `onBell` | `TerminalEvent::Bell` | ✅ Complete | Via event system |
| `onTitleChange` | `TerminalEvent::TitleChanged` | ✅ Complete | |
| `clear()` | Manual via screen API | ⚠️ Partial | Need helper method |
| `reset()` | `Terminal::new()` | ⚠️ Workaround | Need proper reset |
| `options.theme` | `ColorPalette` | ✅ Complete | Via set_palette |
| `options.scrollback` | `TerminalConfig.scrollback_lines` | ✅ Complete | |
| `options.cursorBlink` | `TerminalConfig.cursor_blink` | ✅ Complete | |
| `options.cursorStyle` | `TerminalConfig.cursor_shape` | ✅ Complete | |

### ❌ Critical Missing in libvt

| xterm.js API | libvt Status | Priority | Effort |
|--------------|--------------|----------|--------|
| `buffer.active` | ❌ No alt screen | 🔴 CRITICAL | 1 week |
| `buffer.normal` | ❌ No alt screen | 🔴 CRITICAL | |
| `buffer.alternate` | ❌ No alt screen | 🔴 CRITICAL | |
| `onBufferChange` | ❌ No alt screen | 🔴 CRITICAL | |
| `onData` | ❌ No input event | 🔴 CRITICAL | 3 days |
| `onBinary` | ❌ No binary mode | 🟡 HIGH | 1 day |
| `onKey` | ❌ No key event | 🔴 CRITICAL | 3 days |
| `paste()` | ❌ No bracketed paste impl | 🔴 CRITICAL | 3 days |
| `input()` | ❌ No high-level input | 🟡 HIGH | 1 week |
| `select()` | ⚠️ Basic only | 🔴 CRITICAL | 1 week |
| `selectAll()` | ❌ Missing | 🟡 HIGH | 2 days |
| `selectLines()` | ❌ Missing | 🟡 HIGH | 2 days |
| `getSelection()` | ⚠️ Basic only | 🔴 CRITICAL | 3 days |
| `hasSelection()` | ⚠️ Can check Option | ✅ Trivial | 1 hour |
| `clearSelection()` | ✅ Exists | ✅ Complete | - |
| `scrollLines()` | ❌ Missing | 🟡 HIGH | 3 days |
| `scrollPages()` | ❌ Missing | 🟡 HIGH | 1 day |
| `scrollToTop()` | ❌ Missing | 🟡 HIGH | 1 day |
| `scrollToBottom()` | ❌ Missing | 🟡 HIGH | 1 day |
| `scrollToLine()` | ❌ Missing | 🟡 HIGH | 1 day |
| `focus()` | ❌ N/A (no DOM) | ℹ️ N/A | - |
| `blur()` | ❌ N/A (no DOM) | ℹ️ N/A | - |
| `resize()` | ✅ Exists | ✅ Complete | - |
| `registerMarker()` | ❌ Missing | 🟠 MEDIUM | 1 week |
| `markers` | ❌ Missing | 🟠 MEDIUM | |
| `parser.registerCsiHandler()` | ❌ Missing | 🟡 HIGH | 2 weeks |
| `parser.registerOscHandler()` | ❌ Missing | 🟡 HIGH | |
| `unicode.activeVersion` | ❌ Missing | 🟠 MEDIUM | 1 week |
| `modes` | ⚠️ Partial | 🟡 HIGH | 2 weeks |

### ⚠️ Rendering (Not Applicable to libvt)

These are **frontend concerns**, not terminal emulator concerns:

| xterm.js API | libvt Position | Notes |
|--------------|----------------|-------|
| `element` | N/A | DOM-specific, handled by app |
| `textarea` | N/A | Input element, handled by app |
| `open(parent)` | N/A | Attaching to DOM, app responsibility |
| `refresh()` | N/A | Rendering trigger, app decides |
| `clearTextureAtlas()` | N/A | WebGL optimization, renderer concern |
| `onRender` | N/A | Render loop event, app handles |
| `onCursorMove` | ✅ Can emit | Optional event for cursor tracking |
| `onScroll` | ❌ Need to add | Event when scrollback changes |
| `loadAddon()` | ❌ Different model | libvt: trait-based extensions |

---

## VSCode Integration Architecture

### Current: xterm.js + node-pty

```
┌─────────────────────────────────────────────────┐
│             VSCode (Electron)                    │
│  ┌───────────────────────────────────────────┐  │
│  │  Terminal View (TypeScript)                │  │
│  │  ┌─────────────────────────────────────┐  │  │
│  │  │  xterm.js (DOM rendering)           │  │  │
│  │  │  - Canvas/WebGL layers              │  │  │
│  │  │  - Input handling (textarea)        │  │  │
│  │  │  - Selection UI                     │  │  │
│  │  └──────────┬──────────────────────────┘  │  │
│  │             │ write(), onData, onResize    │  │
│  │  ┌──────────▼──────────────────────────┐  │  │
│  │  │  Terminal Process (TypeScript)      │  │  │
│  │  │  - Manages terminal lifecycle       │  │  │
│  │  │  - Bridges xterm.js ↔ pty          │  │  │
│  │  └──────────┬──────────────────────────┘  │  │
│  └─────────────┼───────────────────────────────┘
│                │                                 │
│  ┌─────────────▼──────────────────────────┐     │
│  │  node-pty (Native C++ Node Module)     │     │
│  │  - Spawns shell process                │     │
│  │  - PTY management                       │     │
│  └─────────────┬──────────────────────────┘     │
└────────────────┼──────────────────────────────────┘
                 │
                 ▼
          ┌──────────────┐
          │ Shell Process│
          │ (bash, zsh)  │
          └──────────────┘
```

**Data Flow**:
1. Shell → node-pty → Terminal Process → xterm.js.write()
2. User input → xterm.js.onData → Terminal Process → node-pty → Shell

### Proposed: libvt (Rust) + NAPI-RS

```
┌─────────────────────────────────────────────────┐
│             VSCode (Electron)                    │
│  ┌───────────────────────────────────────────┐  │
│  │  Terminal View (TypeScript)                │  │
│  │  ┌─────────────────────────────────────┐  │  │
│  │  │  Canvas Renderer (TypeScript/WASM)  │  │  │
│  │  │  - Renders cells from libvt buffer  │  │  │
│  │  │  - Input handling (textarea/Canvas) │  │  │
│  │  │  - Selection overlay                │  │  │
│  │  └──────────┬──────────────────────────┘  │  │
│  │             │ getCells(), onDirty, etc     │  │
│  │  ┌──────────▼──────────────────────────┐  │  │
│  │  │  NAPI-RS Bridge (Rust→JS)           │  │  │
│  │  │  - Exposes libvt::Terminal to JS    │  │  │
│  │  │  - Zero-copy buffer access          │  │  │
│  │  │  - Event callbacks (onData, etc)    │  │  │
│  │  └──────────┬──────────────────────────┘  │  │
│  └─────────────┼───────────────────────────────┘
│                │                                 │
│  ┌─────────────▼──────────────────────────┐     │
│  │  libvt Terminal (Rust Native Module)   │     │
│  │  - Escape sequence parsing             │     │
│  │  - Terminal state machine              │     │
│  │  - Buffer management                   │     │
│  │  - Input encoding                      │     │
│  └─────────────┬──────────────────────────┘     │
│                │                                 │
│  ┌─────────────▼──────────────────────────┐     │
│  │  portable-pty or node-pty              │     │
│  │  - Spawns shell process                │     │
│  │  - PTY management                       │     │
│  └─────────────┬──────────────────────────┘     │
└────────────────┼──────────────────────────────────┘
                 │
                 ▼
          ┌──────────────┐
          │ Shell Process│
          │ (bash, zsh)  │
          └──────────────┘
```

**Key Differences**:
1. **Terminal state** in Rust (libvt) instead of TypeScript (xterm.js)
2. **Renderer** is separate (TypeScript or WASM)
3. **NAPI-RS** provides zero-copy bridge
4. **PTY** remains external (same as xterm.js)

---

## NAPI-RS Bridge API Design

### TypeScript API (matches xterm.js)

```typescript
// TypeScript side (VSCode extension)
import { Terminal } from 'libvt-napi';

// Create terminal (matches xterm.js)
const term = new Terminal({
    cols: 80,
    rows: 24,
    scrollback: 10000,
    cursorBlink: true,
    cursorShape: 'block'
});

// Write data (matches xterm.js)
term.write('Hello \x1b[1;32mWorld\x1b[0m\n');

// Event handlers (matches xterm.js)
term.onData((data) => {
    pty.write(data);  // Send to shell
});

term.onBell(() => {
    // Visual bell
});

term.onTitleChange((title) => {
    // Update window title
});

term.onResize(({ cols, rows }) => {
    pty.resize(cols, rows);
});

// Buffer access (matches xterm.js)
const buffer = term.buffer.active;
for (let row = 0; row < term.rows; row++) {
    const line = buffer.getLine(row);
    if (line) {
        const text = line.translateToString();
        console.log(text);
    }
}

// Selection (matches xterm.js)
term.selectAll();
const selectedText = term.getSelection();

// Scrolling (matches xterm.js)
term.scrollLines(-5);  // Scroll up 5 lines
term.scrollToTop();

// Resize (matches xterm.js)
term.resize(100, 30);

// Disposal (matches xterm.js)
term.dispose();
```

### Rust NAPI-RS Implementation

```rust
// src/lib.rs - NAPI-RS bindings
use napi::{bindgen_prelude::*, JsFunction};
use napi_derive::napi;
use libvt::{Terminal as LibVtTerminal, TerminalConfig, KeyCode, KeyModifiers};

#[napi(object)]
pub struct TerminalOptions {
    pub cols: u32,
    pub rows: u32,
    pub scrollback: Option<u32>,
    pub cursor_blink: Option<bool>,
    pub cursor_shape: Option<String>,
}

#[napi]
pub struct Terminal {
    inner: LibVtTerminal,
    on_data_callback: Option<JsFunction>,
    on_bell_callback: Option<JsFunction>,
    on_title_change_callback: Option<JsFunction>,
    on_resize_callback: Option<JsFunction>,
}

#[napi]
impl Terminal {
    #[napi(constructor)]
    pub fn new(options: TerminalOptions) -> Result<Self> {
        let config = TerminalConfig {
            scrollback_lines: options.scrollback.unwrap_or(10000) as usize,
            cursor_blink: options.cursor_blink.unwrap_or(true),
            ..Default::default()
        };

        let inner = LibVtTerminal::new(
            options.cols as u16,
            options.rows as u16,
            config
        );

        Ok(Terminal {
            inner,
            on_data_callback: None,
            on_bell_callback: None,
            on_title_change_callback: None,
            on_resize_callback: None,
        })
    }

    #[napi]
    pub fn write(&mut self, data: Either<String, Buffer>) -> Result<()> {
        let bytes = match data {
            Either::A(s) => s.as_bytes(),
            Either::B(b) => &b,
        };
        self.inner.write(bytes);

        // Process events and trigger callbacks
        self.process_events()?;

        Ok(())
    }

    #[napi]
    pub fn on_data(&mut self, callback: JsFunction) -> Result<()> {
        self.on_data_callback = Some(callback);
        Ok(())
    }

    #[napi]
    pub fn on_bell(&mut self, callback: JsFunction) -> Result<()> {
        self.on_bell_callback = Some(callback);
        Ok(())
    }

    #[napi]
    pub fn on_title_change(&mut self, callback: JsFunction) -> Result<()> {
        self.on_title_change_callback = Some(callback);
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

    #[napi(getter)]
    pub fn buffer(&self) -> Buffer {
        Buffer::new(self)
    }

    #[napi]
    pub fn resize(&mut self, cols: u32, rows: u32) -> Result<()> {
        self.inner.resize(cols as u16, rows as u16);

        // Trigger onResize callback
        if let Some(ref callback) = self.on_resize_callback {
            // Call JS callback with {cols, rows}
        }

        Ok(())
    }

    #[napi]
    pub fn select_all(&mut self) -> Result<()> {
        let (cols, rows) = self.inner.size();
        self.inner.set_selection(
            Position { col: 0, row: 0 },
            Position { col: cols - 1, row: rows - 1 }
        );
        Ok(())
    }

    #[napi]
    pub fn get_selection(&self) -> Option<String> {
        self.inner.selected_text()
    }

    #[napi]
    pub fn clear_selection(&mut self) {
        self.inner.clear_selection();
    }

    // ... more methods

    fn process_events(&mut self) -> Result<()> {
        let events = self.inner.take_events();

        for event in events {
            match event {
                TerminalEvent::Bell => {
                    if let Some(ref callback) = self.on_bell_callback {
                        callback.call_without_args(None)?;
                    }
                }
                TerminalEvent::TitleChanged(title) => {
                    if let Some(ref callback) = self.on_title_change_callback {
                        callback.call(None, &[title.into()])?;
                    }
                }
                // ... more events
            }
        }

        Ok(())
    }
}

#[napi]
pub struct Buffer {
    // Reference to terminal
}

#[napi]
impl Buffer {
    #[napi]
    pub fn get_line(&self, row: u32) -> Option<BufferLine> {
        // Access terminal.inner.get_line(row)
        // Convert to BufferLine
    }

    #[napi(getter)]
    pub fn cursor_x(&self) -> u32 {
        // terminal.cursor_position().0
    }

    #[napi(getter)]
    pub fn cursor_y(&self) -> u32 {
        // terminal.cursor_position().1
    }
}

#[napi]
pub struct BufferLine {
    // Line data
}

#[napi]
impl BufferLine {
    #[napi]
    pub fn translate_to_string(&self, trim_right: Option<bool>) -> String {
        // Convert line to string
    }

    #[napi]
    pub fn get_cell(&self, col: u32) -> Option<BufferCell> {
        // Get cell at column
    }
}

#[napi]
pub struct BufferCell {
    // Cell data
}

#[napi]
impl BufferCell {
    #[napi]
    pub fn get_chars(&self) -> String {
        // cell.text()
    }

    #[napi]
    pub fn is_bold(&self) -> bool {
        // cell.attrs().bold
    }

    // ... more cell methods
}
```

---

## Implementation Roadmap

### Phase 1: Core xterm.js API Compatibility (8-10 weeks)

**Week 1-2: Event System**
- ✅ Add `onData` event (keyboard/paste input)
- ✅ Add `onBinary` event (binary mode)
- ✅ Add `onKey` event (raw key events)
- ✅ Add `onResize` event
- ✅ Add `onScroll` event
- ✅ Add `onCursorMove` event
- ✅ Add `onSelectionChange` event

**Week 3: Alternate Screen Buffer** 🔴 CRITICAL
- ✅ Add second screen buffer
- ✅ CSI ?1049h/l switching
- ✅ Buffer switch events
- ✅ State preservation

**Week 4-5: Selection Enhancement**
- ✅ Block/rectangular selection
- ✅ `selectAll()` method
- ✅ `selectLines(start, end)` method
- ✅ `hasSelection()` method
- ✅ `getSelectionPosition()` returning BufferRange

**Week 6: Scrolling API**
- ✅ `scrollLines(amount)` method
- ✅ `scrollPages(pageCount)` method
- ✅ `scrollToTop()` method
- ✅ `scrollToBottom()` method
- ✅ `scrollToLine(line)` method
- ✅ `onScroll` event

**Week 7: Input Handling**
- ✅ `paste(data)` method (bracketed paste)
- ✅ `input(data, wasUserInput)` method
- ✅ Binary mode support
- ✅ Custom key/wheel event handlers

**Week 8: Markers & Decorations**
- ✅ `registerMarker(cursorYOffset)` API
- ✅ Marker tracking during scroll
- ✅ Marker disposal
- ✅ `markers` property

**Week 9-10: Parser Extension API** 🟡 HIGH
- ✅ `parser.registerCsiHandler()`
- ✅ `parser.registerOscHandler()`
- ✅ `parser.registerEscHandler()`
- ✅ `parser.registerDcsHandler()`
- ✅ Custom escape sequence handlers

### Phase 2: Buffer Access API (3-4 weeks)

**Week 11-12: IBuffer Interface**
- ✅ `buffer.active` property
- ✅ `buffer.normal` property
- ✅ `buffer.alternate` property
- ✅ `buffer.cursorX/cursorY` getters
- ✅ `buffer.viewportY` (scroll position)
- ✅ `buffer.baseY` (scrollback top)
- ✅ `buffer.length` (total lines)
- ✅ `buffer.getLine(y)` method
- ✅ `buffer.getNullCell()` method

**Week 13-14: IBufferLine & IBufferCell**
- ✅ `line.isWrapped` property
- ✅ `line.length` property
- ✅ `line.getCell(x)` method
- ✅ `line.translateToString()` method
- ✅ Full `IBufferCell` interface (getChars, getWidth, isBold, etc.)

### Phase 3: Advanced Features (6-8 weeks)

**Week 15-16: Unicode Handling**
- ✅ `unicode.activeVersion` property
- ✅ `unicode.versions` list
- ✅ Grapheme cluster support
- ✅ Unicode version switching

**Week 17-18: Modes API**
- ✅ `modes.applicationCursorKeysMode`
- ✅ `modes.applicationKeypadMode`
- ✅ `modes.bracketedPasteMode`
- ✅ `modes.insertMode`
- ✅ `modes.mouseTrackingMode`
- ✅ `modes.originMode`
- ✅ `modes.reverseWraparoundMode`
- ✅ `modes.sendFocusMode`
- ✅ `modes.wraparoundMode`

**Week 19-20: Link Provider API**
- ✅ `registerLinkProvider()` interface
- ✅ Custom link detection
- ✅ Hover callbacks
- ✅ Activation callbacks

**Week 21-22: Character Joiner API**
- ✅ `registerCharacterJoiner()` for ligatures
- ✅ Custom character combining
- ✅ Performance optimization

### Phase 4: NAPI-RS Bindings (4-6 weeks)

**Week 23-24: Basic NAPI-RS Structure**
- ✅ Terminal class wrapper
- ✅ write/writeln methods
- ✅ Event callback infrastructure
- ✅ rows/cols getters
- ✅ resize method

**Week 25-26: Buffer Access Bindings**
- ✅ Buffer/BufferLine/BufferCell classes
- ✅ Zero-copy buffer access where possible
- ✅ TypeScript type definitions (.d.ts)

**Week 27-28: Complete API Bindings**
- ✅ All selection methods
- ✅ All scrolling methods
- ✅ All event handlers
- ✅ Parser extension bindings
- ✅ Options/config bindings

### Phase 5: VSCode Integration (4-6 weeks)

**Week 29-30: Canvas Renderer (TypeScript)**
- ✅ 4-layer rendering (text, selection, links, cursor)
- ✅ Damage region tracking
- ✅ Font atlas/glyph caching
- ✅ Color theme support

**Week 31-32: Input Handling**
- ✅ Textarea for composition
- ✅ Keyboard event translation
- ✅ Mouse event handling
- ✅ IME support

**Week 33-34: VSCode Extension**
- ✅ Terminal provider implementation
- ✅ Theme integration
- ✅ Configuration sync
- ✅ Extension packaging

---

## Performance Comparison

### xterm.js (JavaScript)

**Pros**:
- ✅ Well-optimized (10+ years of tuning)
- ✅ WebGL renderer (GPU acceleration)
- ✅ Lazy evaluation
- ✅ Incremental DOM updates

**Cons**:
- ❌ JavaScript overhead (~10-20% of cycles on GC)
- ❌ String immutability (lots of allocations)
- ❌ No SIMD vectorization
- ❌ JIT warmup time

**Benchmarks** (typical):
- Parse throughput: ~30-50 MB/s
- Scrollback: ~100k lines comfortable
- Memory: ~100-200 MB for 100k scrollback

### libvt (Rust)

**Expected Pros**:
- ✅ Zero-cost abstractions
- ✅ No GC pauses (predictable latency)
- ✅ SIMD opportunities (portable_simd)
- ✅ Better cache locality
- ✅ Compile-time optimizations

**Expected Cons**:
- ❌ NAPI-RS bridge overhead (serialization)
- ❌ No lazy evaluation yet
- ❌ Not GPU-accelerated (renderer is separate)

**Expected Benchmarks** (target):
- Parse throughput: ~100-200 MB/s (2-4x faster)
- Scrollback: 500k+ lines comfortable
- Memory: ~50-100 MB for 100k scrollback (2x better)

**Key Advantage**: Predictable latency (no GC pauses = smooth 60 FPS)

---

## Compatibility Strategy

### Option A: Drop-in Replacement (High Effort)

**Goal**: 100% API-compatible with xterm.js

**Pros**:
- ✅ VSCode can switch with minimal changes
- ✅ Existing xterm.js extensions work
- ✅ Familiar API for developers

**Cons**:
- ❌ Must implement ALL xterm.js APIs (55+ methods, 12 events, 40 options)
- ❌ Must match xterm.js quirks/bugs for compatibility
- ❌ NAPI-RS bridge adds complexity

**Timeline**: ~30-34 weeks (Phase 1-5 above)

### Option B: Rust-Native API (Low Effort)

**Goal**: Purpose-built Rust API, VSCode adapts

**Pros**:
- ✅ Clean, idiomatic Rust API
- ✅ Faster to implement (reuse existing libvt)
- ✅ Better performance (less marshalling)

**Cons**:
- ❌ VSCode needs custom integration code
- ❌ Not compatible with existing xterm.js ecosystem
- ❌ Higher adoption barrier

**Timeline**: ~12-16 weeks (just Phase 4-5)

### Option C: Hybrid Approach (RECOMMENDED)

**Goal**: Core Rust API + xterm.js compatibility layer

**Architecture**:
```
┌────────────────────────────────────────┐
│  xterm.js Compatibility Layer (TS)     │
│  - Matches xterm.js Terminal API       │
│  - Event translation                   │
│  - Option mapping                      │
└──────────────┬─────────────────────────┘
               │
┌──────────────▼─────────────────────────┐
│  NAPI-RS Bridge (Minimal)              │
│  - Simple get/set methods              │
│  - Direct buffer access                │
│  - Event callbacks                     │
└──────────────┬─────────────────────────┘
               │
┌──────────────▼─────────────────────────┐
│  libvt Core (Rust)                     │
│  - Clean, idiomatic API                │
│  - High performance                    │
└────────────────────────────────────────┘
```

**Pros**:
- ✅ Best of both worlds
- ✅ Faster initial implementation
- ✅ Can optimize over time
- ✅ Compatibility layer is pure TypeScript (easier to debug)

**Cons**:
- ⚠️ Extra layer of abstraction
- ⚠️ Some performance cost in translation

**Timeline**: ~20-24 weeks

---

## Comparison Summary

| Aspect | xterm.js | libvt (Target) |
|--------|----------|----------------|
| **Language** | TypeScript | Rust |
| **LOC** | ~50,000+ | ~10,000 (est) |
| **API Surface** | 55+ methods, 12 events | Similar (via NAPI) |
| **Addons** | 13 official | N/A (trait-based) |
| **Rendering** | Canvas/WebGL | External (app) |
| **Performance** | 30-50 MB/s parse | 100-200 MB/s (est) |
| **Memory** | 100-200 MB | 50-100 MB (est) |
| **GC Pauses** | Yes (~10ms) | No |
| **SIMD** | No | Yes (portable_simd) |
| **Safety** | TypeScript | 100% safe Rust |
| **Maturity** | 10+ years | Prototype |
| **Adoption** | VSCode, Hyper, etc. | None yet |

---

## Recommendation

**For VSCode Integration**:

1. **Phase 1** (8-10 weeks): Core xterm.js API compatibility
   - Events system (onData, onBinary, etc.)
   - Alternate screen buffer
   - Selection enhancement
   - Scrolling API
   - Input handling
   - Markers

2. **Phase 2** (3-4 weeks): Buffer access API
   - IBuffer, IBufferLine, IBufferCell interfaces
   - Zero-copy where possible

3. **Phase 3** (6-8 weeks): Advanced features
   - Unicode handling
   - Modes API
   - Parser extension (custom handlers)
   - Link provider

4. **Phase 4** (4-6 weeks): NAPI-RS bindings
   - TypeScript-compatible API
   - Event callbacks
   - Type definitions

5. **Phase 5** (4-6 weeks): Renderer + VSCode integration
   - Canvas/WebGL renderer (TypeScript)
   - VSCode extension

**Total Timeline**: ~25-34 weeks (6-8 months)

**Minimum Viable** (Phase 1-2 only): ~11-14 weeks (3-4 months)

**MVP would support**:
- Basic terminal emulation (write, read, events)
- Alternate screen (vim, less)
- Selection (copy/paste)
- Scrolling
- Enough for basic VSCode terminal use

---

## Next Steps

1. **Validate approach with VSCode team** - Ensure NAPI-RS + Rust approach is acceptable
2. **Implement Phase 1** - Core API compatibility (11-14 weeks)
3. **Build proof-of-concept** - Simple VSCode extension with canvas renderer
4. **Benchmark** - Verify performance gains vs xterm.js
5. **Iterate** - Add Phase 2-5 features based on feedback

---

**Document Version**: 1.0
**Last Updated**: 2025-11-18
**Based on**: xterm.js 5.x API, libvt v0.1.0
