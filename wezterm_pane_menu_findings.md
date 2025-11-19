# WezTerm Pane Management and macOS Menu Architecture - Comprehensive Findings

## Executive Summary
This document provides a thorough analysis of WezTerm's existing pane management system and macOS menu implementation, which are critical for understanding where native context menus should be integrated.

---

## 1. PANE MANAGEMENT ARCHITECTURE

### 1.1 Core Pane Structures (mux/src/)

#### **Pane Trait Definition** (`mux/src/pane.rs`)
- **Type**: `PaneId` = `usize` (allocated via `alloc_pane_id()`)
- **Key Methods**:
  - `pane_id()` - returns unique identifier
  - `get_config()` / `set_config()` - terminal configuration
  - `is_mouse_grabbed()` - check if pane captures mouse
  - `perform_assignment()` - handle key assignments
  - `search()`, `get_lines()` - content access

**Key Trait Methods for Integration Points**:
```rust
pub trait Pane: Downcast + Send {
    fn pane_id(&self) -> PaneId;
    fn perform_assignment(&self, assignment: &KeyAssignment, context: &dyn PaneEmbedderState) -> PerformAssignmentResult;
    fn mouse_event(&mut self, event: MouseEvent) -> PerformAssignmentResult;
}
```

#### **Tab Structure** (`mux/src/tab.rs`)
- **Container**: Tab holds a `Tree` (binary tree) of panes
- **Split Structure**: Uses `bintree::Tree<Arc<dyn Pane>, SplitDirectionAndSize>`
- **Key Classes**:
  ```rust
  pub struct Tab {
      inner: Mutex<TabInner>,
      tab_id: TabId,
  }

  struct TabInner {
      id: TabId,
      pane: Option<Tree>,        // Binary tree of panes
      size: TerminalSize,
      active: usize,              // Index of active pane
      zoomed: Option<Arc<dyn Pane>>, // Zoomed pane if present
      recency: Recency,           // Track pane access order
  }
  ```

#### **Split Representation** (`mux/src/tab.rs`)
```rust
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

pub struct SplitDirectionAndSize {
    pub direction: SplitDirection,
    pub first: TerminalSize,
    pub second: TerminalSize,
}

pub struct PositionedPane {
    pub index: usize,           // Topological index
    pub is_active: bool,
    pub is_zoomed: bool,
    pub left: usize,            // Cell coordinates
    pub top: usize,
    pub width: usize,
    pub height: usize,
    pub pane: Arc<dyn Pane>,
}

pub struct PositionedSplit {
    pub index: usize,           // Node index
    pub direction: SplitDirection,
    pub left: usize,            // Cell coordinates
    pub top: usize,
    pub size: usize,            // Width for vertical, height for horizontal
}
```

### 1.2 Split Operations

#### **Core Split API** (`mux/src/tab.rs:740`)
```rust
pub fn split_and_insert(
    &self,
    pane_index: usize,
    request: SplitRequest,
    pane: Arc<dyn Pane>,
) -> anyhow::Result<usize>
```

#### **SplitRequest Structure**
```rust
pub struct SplitRequest {
    pub direction: SplitDirection,      // Horizontal or Vertical
    pub target_is_second: bool,         // New pane position (left/right or top/bottom)
    pub top_level: bool,                // Apply to tab root vs active pane
    pub size: SplitSize,                // Cells or Percent
}
```

#### **High-Level Split** (`mux/src/lib.rs:1187`)
```rust
pub async fn split_pane(
    &self,
    pane_id: PaneId,
    request: SplitRequest,
    source: SplitSource,
    domain: config::keyassignment::SpawnTabDomain,
) -> anyhow::Result<(Arc<dyn Pane>, TerminalSize)>
```

**Key Operations**:
- Resolves pane_id to domain, window_id, tab_id
- Attaches domain if needed
- Delegates to domain's `split_pane()` method
- Returns new pane and its terminal size

### 1.3 Related Pane Operations (`mux/src/lib.rs`)
```rust
pub async fn move_pane_to_new_tab(
    &self,
    pane_id: PaneId,
    window_id: Option<WindowId>,
    workspace_for_new_window: Option<String>,
) -> anyhow::Result<(Arc<Tab>, WindowId)>
```

---

## 2. macOS MENU ARCHITECTURE

### 2.1 Menu Core (`window/src/os/macos/menu.rs`)

#### **Menu Structure**
```rust
pub struct Menu {
    menu: StrongPtr,  // Wraps NSMenu*
}

pub struct MenuItem {
    item: StrongPtr,  // Wraps NSMenuItem*
}
```

#### **Menu Methods**
- `new_with_title(title: &str)` - Create menu
- `assign_as_main_menu()` - Set as app menu bar
- `assign_as_windows_menu()` - Set as Windows menu
- `assign_as_app_menu()` - Set as WezTerm menu
- `assign_as_help_menu()` - Set as Help menu
- `assign_as_services_menu()` - Set as Services menu
- `get_or_create_sub_menu(title: &str, on_create: F)` - Submenu handling
- `add_item(&MenuItem)`, `remove_item(&MenuItem)`
- `items()` - Iterate all items

#### **MenuItem Methods**
- `new_with(title: &str, action: Option<SEL>, key: &str)` - Create item
- `new_separator()` - Separator item
- `set_sub_menu(&Menu)`, `get_sub_menu()` - Submenu association
- `set_represented_item(RepresentedItem)` - Attach data to menu item
- `get_represented_item()` - Retrieve attached data
- `set_key_equivalent()`, `set_key_equiv_modifier_mask()` - Keyboard shortcuts

#### **RepresentedItem Wrapper** (for attaching KeyAssignments to menu items)
```rust
#[derive(Clone, Debug, PartialEq)]
pub enum RepresentedItem {
    KeyAssignment(KeyAssignment),
}
```
- Wrapped in Objective-C class `WezTermNSMenuRepresentedItem`
- Uses instance variable `WRAPPER_FIELD_NAME` = "item"
- Allows KeyAssignment to be associated with NSMenuItem via `setRepresentedObject:`

### 2.2 Menu Bar Recreation (`wezterm-gui/src/commands.rs:380`)

#### **macOS-Specific Implementation**
```rust
#[cfg(target_os = "macos")]
pub fn recreate_menubar(config: &ConfigHandle) {
    // Key functions:
    // 1. Get or create main menu
    // 2. Build submenu hierarchy from commands
    // 3. Update existing items or create new ones
    // 4. Use tag-based garbage collection to remove orphaned items
}

#[cfg(not(target_os = "macos"))]
pub fn recreate_menubar(_config: &ConfigHandle) {}  // No-op on other platforms
```

#### **Menu Hierarchy**
```
Preferred Menu Order: ["WezTerm", "Shell", "Edit", "View", "Window"]
Menu Structure:
  MainMenu
  ├── WezTerm (assign_as_app_menu)
  │   ├── About WezTerm
  │   ├── Services submenu (assign_as_services_menu)
  │   └── (standard app items)
  ├── Shell
  │   ├── [Launch menu items]
  │   ├── Domain attachment/detachment items
  │   └── Workspace switching items
  ├── Edit
  ├── View
  ├── Window (assign_as_windows_menu)
  │   └── [Auto-populated by macOS]
  └── Help (assign_as_help_menu)
```

#### **Menu Item Handling** (`window/src/os/macos/app.rs`)
```rust
// Menu item action selector
extern "C" fn wezterm_perform_key_assignment(
    _self: &mut Object,
    _sel: Sel,
    menu_item: *mut Object,
) {
    let menu_item = MenuItem::with_menu_item(menu_item);
    let action = menu_item.get_represented_item();  // Extract KeyAssignment
    if let Some(RepresentedItem::KeyAssignment(action)) = action {
        if let Some(conn) = Connection::get() {
            conn.dispatch_app_event(ApplicationEvent::PerformKeyAssignment(action));
        }
    }
}
```

### 2.3 Dock Menu (`window/src/os/macos/app.rs:131`)

```rust
extern "C" fn application_dock_menu(
    _self: &mut Object,
    _sel: Sel,
    _app: *mut Object,
) -> *mut Object {
    let dock_menu = Menu::new_with_title("");
    let new_window_item = MenuItem::new_with(
        "New Window",
        Some(sel!(weztermPerformKeyAssignment:)),
        ""
    );
    new_window_item.set_represented_item(
        RepresentedItem::KeyAssignment(KeyAssignment::SpawnWindow)
    );
    dock_menu.add_item(&new_window_item);
    dock_menu.autorelease()
}
```

---

## 3. GUI PANE RENDERING AND INTERACTION

### 3.1 GUI Pane Structures (`wezterm-gui/src/termwindow/mod.rs`)

#### **UI Items** (interactive elements)
```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UIItemType {
    TabBar(usize),          // Tab index
    CloseTab(TabId),
    Split(PositionedSplit), // Split divider
    AboveScrollThumb,
    BelowScrollThumb,
    ScrollThumb,
}

pub struct UIItem {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub item_type: UIItemType,
}
```

### 3.2 Split Rendering (`wezterm-gui/src/termwindow/render/split.rs`)

```rust
pub fn paint_split(
    &mut self,
    layers: &mut TripleLayerQuadAllocator,
    split: &PositionedSplit,
    pane: &Arc<dyn Pane>,
) -> anyhow::Result<()>
```

**Split Rendering Logic**:
- Horizontal splits: Vertical line at `split.left` position, height = `split.size`
- Vertical splits: Horizontal line at `split.top` position, width = `split.size`
- Creates `UIItem::Split` for hit-testing and interaction
- Color from `pane.palette().split`

### 3.3 Mouse Event Handling (`wezterm-gui/src/termwindow/mouseevent.rs`)

#### **Key Handler**
```rust
pub fn mouse_event_impl(&mut self, event: MouseEvent, context: &dyn WindowOps)
```

#### **Right-Click Handling**
- Line 411: `MousePress::Right => Some(KeyAssignment::ShowLauncher)` (tab bar context)
- Line 525: `WMEK::Press(MousePress::Right)` handling for different UI items
- Line 530: `do_new_tab_button_click(MousePress::Right)` dispatches ShowLauncher

#### **UI Item Hit Testing**
```rust
fn resolve_ui_item(&self, event: &MouseEvent) -> Option<UIItem> {
    let x = event.coords.x;
    let y = event.coords.y;
    self.ui_items
        .iter()
        .rev()  // Top-to-bottom, reverse order for z-index
        .find(|item| item.hit_test(x, y))
        .cloned()
}
```

#### **Current Context Menu** (via ShowLauncher)
- Triggered on right-click of new-tab button
- Shows launcher overlay (not native context menu)
- Line 525-530: Tab bar right-click dispatch

---

## 4. COMMAND SYSTEM AND KEY ASSIGNMENTS

### 4.1 Command Definition (`wezterm-gui/src/commands.rs`)

#### **CommandDef Structure**
```rust
pub struct CommandDef {
    pub brief: Cow<'static, str>,           // Menu label
    pub doc: Cow<'static, str>,             // Description
    pub keys: Vec<(Modifiers, String)>,     // Keyboard shortcuts
    pub args: &'static [ArgType],           // Context: ActivePane, ActiveTab, ActiveWindow
    pub menubar: &'static [&'static str],   // Menu path: ["Edit", "Copy"] -> Edit > Copy
    pub icon: Option<&'static str>,         // Icon identifier
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ArgType {
    ActivePane,
    ActiveTab,
    ActiveWindow,
}
```

#### **ExpandedCommand** (resolved at runtime)
```rust
pub struct ExpandedCommand {
    pub brief: Cow<'static, str>,
    pub doc: Cow<'static, str>,
    pub action: KeyAssignment,              // The actual action to perform
    pub keys: Vec<(Modifiers, KeyCode)>,
    pub menubar: &'static [&'static str],
    pub icon: Option<Cow<'static, str>>,
}
```

### 4.2 Relevant KeyAssignments for Pane Operations

From `config/src/keyassignment.rs` (inferred from usage):
- `SplitPane(SplitSize)` - Split active pane
- `SplitHorizontally`
- `SplitVertically`
- `AdjustPaneSize(PaneDirection, usize)` - Resize pane
- `RotatePanes`
- `MoveTabToNewWindow`
- `MovePane`
- `ActivatePaneDirection(PaneDirection)`
- `TogglePaneZoomState`

---

## 5. METAL RENDERER FOR macOS (`wezterm-metal/src/`)

### 5.1 Core Components
- `renderer.rs` - High-performance Metal renderer
- `atlas.rs` - Glyph atlas management
- `blit.rs` - GPU blitting with dirty rect tracking
- `pipeline.rs` - Metal rendering pipeline
- `debug.rs` / `debug_overlay.rs` - Performance profiling

### 5.2 Architecture
```rust
pub struct MetalRenderer {
    // GPU-accelerated rendering
}

pub struct RenderConfig {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub gpu_time_ms: f32,
    // ...
}

pub struct PerformanceMetrics {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub gpu_time_ms: f32,
    pub draw_calls: u32,
    pub glyphs_rendered: u32,
    pub atlas_memory_mb: f32,
}
```

**Relevant for Context Menus**: Metal renderer provides GPU-accelerated rendering context, but context menus should use native macOS NSMenu for consistency.

---

## 6. APPLICATION EVENT DISPATCH

### 6.1 Event Flow (`wezterm-gui/src/frontend.rs`)

```rust
pub fn app_event_handler(event: ApplicationEvent) {
    // Routes events to appropriate handler
    // Menu items dispatch: ApplicationEvent::PerformKeyAssignment(KeyAssignment)
}
```

### 6.2 Event Types from Menu System
- `ApplicationEvent::PerformKeyAssignment(KeyAssignment)` - From menu/dock menus
- `ApplicationEvent::OpenCommandScript(filename)` - File open from Dock
- `ApplicationEvent::PerformKeyAssignment(KeyAssignment::SpawnWindow)` - New window from Dock

---

## 7. INTEGRATION POINTS FOR CONTEXT MENUS

### 7.1 **RIGHT-CLICK ON PANE** (NEW FEATURE)

**Current Path**:
- `mouse_event_impl()` at `wezterm-gui/src/termwindow/mouseevent.rs:61`
- Hit-test UI items: `resolve_ui_item()` line 27
- Currently routes to pane's `mouse_event()` method

**Proposed Integration**:
1. **Detect right-click on pane area**:
   ```rust
   // In mouse_event_impl() when handling pane area
   if event.kind == MouseEventKind::Press(MousePress::Right) {
       // Check if click is on pane (not on split divider)
       // Call new method: show_pane_context_menu(pane_id, x, y)
   }
   ```

2. **Create context menu**:
   ```rust
   fn show_pane_context_menu(
       &mut self,
       pane_id: PaneId,
       x: isize,
       y: isize,
   ) -> anyhow::Result<()> {
       #[cfg(target_os = "macos")]
       {
           let menu = build_pane_context_menu(pane_id);
           // Show NSMenu at mouse position
       }
   }
   ```

3. **Menu items** (examples):
   - Split Pane (Horizontal / Vertical)
   - Move Pane (to new tab, new window)
   - Close Pane
   - Toggle Zoom
   - Paste / Copy (if applicable)

### 7.2 **RIGHT-CLICK ON SPLIT DIVIDER** (NEW FEATURE)

**Current Path**:
- Hit-test returns `UIItem { item_type: UIItemType::Split(...) }`
- Currently unused for interaction (line 46 in mouseevent.rs)

**Proposed Integration**:
1. **Detect right-click on split**:
   ```rust
   if event.kind == MouseEventKind::Press(MousePress::Right) {
       if let Some(UIItem { item_type: UIItemType::Split(split), .. }) = resolved_item {
           show_split_context_menu(split);
       }
   }
   ```

2. **Split context menu**:
   - Swap Panes
   - Resize Options (drag handles)
   - Rotate Panes
   - Equalize Pane Sizes

### 7.3 **RIGHT-CLICK ON TAB BAR** (ENHANCEMENT)

**Current Path**:
- Line 525-530: Handled for new tab button
- Could extend to tab items themselves

**Proposed Menu Items**:
- Close Tab
- Rename Tab
- Detach Tab to New Window
- Move Tab Left / Right

---

## 8. FILE REFERENCES - QUICK LOOKUP

### Core Pane/Tab/Window Management
| File | Purpose | Key Items |
|------|---------|-----------|
| `mux/src/pane.rs` | Pane trait definition | `Pane` trait, `PaneId`, `PerformAssignmentResult` |
| `mux/src/tab.rs` (L:1-250) | Tab structure & split logic | `Tab`, `TabInner`, `SplitDirection`, `SplitRequest`, `PositionedPane`, `PositionedSplit` |
| `mux/src/tab.rs` (L:740) | Split implementation | `split_and_insert()` |
| `mux/src/lib.rs` (L:1187) | High-level split API | `split_pane()` async function |
| `mux/src/window.rs` | Window structure | `Window`, `WindowId`, tab management |
| `mux/src/localpane.rs` | Local pane implementation | Spawn/PTY management |

### macOS Menu System
| File | Purpose | Key Items |
|------|---------|-----------|
| `window/src/os/macos/menu.rs` | Menu/MenuItem wrappers | `Menu`, `MenuItem`, `RepresentedItem` |
| `window/src/os/macos/app.rs` | App delegate, menu handlers | `application_dock_menu()`, `wezterm_perform_key_assignment()` |
| `wezterm-gui/src/commands.rs` (L:380) | Menu bar recreation | `recreate_menubar()` macOS implementation |
| `wezterm-gui/src/commands.rs` (L:56-79) | Command metadata | `CommandDef`, `ExpandedCommand` |

### GUI Rendering & Interaction
| File | Purpose | Key Items |
|------|---------|-----------|
| `wezterm-gui/src/termwindow/mod.rs` | Main window state | `TermWindow`, `UIItemType`, `UIItem` |
| `wezterm-gui/src/termwindow/mouseevent.rs` (L:61) | Mouse event handler | `mouse_event_impl()`, hit testing |
| `wezterm-gui/src/termwindow/mouseevent.rs` (L:411) | Right-click handling | Current ShowLauncher dispatch |
| `wezterm-gui/src/termwindow/render/split.rs` | Split rendering | `paint_split()` |
| `wezterm-gui/src/frontend.rs` | GUI frontend lifecycle | Menu bar recreation calls |

### macOS-Specific
| File | Purpose | Key Items |
|------|---------|-----------|
| `window/src/os/macos/connection.rs` | Platform connection | Event dispatch |
| `window/src/os/macos/window.rs` | Native window implementation | NSWindow handling |
| `window/src/os/macos/metal.rs` | Metal renderer integration | GPU rendering |
| `wezterm-metal/src/lib.rs` | Metal renderer library | `MetalRenderer`, `PerformanceMetrics` |

---

## 9. IMPLEMENTATION ROADMAP FOR CONTEXT MENUS

### Phase 1: Basic Right-Click on Pane
1. Detect right-click in `mouse_event_impl()` (mouseevent.rs:61+)
2. Resolve clicked pane from hit-test
3. Show native NSMenu with basic options:
   - Split Horizontally
   - Split Vertically
   - Close Pane
   - Zoom/Unzoom

### Phase 2: Enhanced Context Menus
1. Right-click on split dividers
2. Right-click on tabs
3. Pane movement options
4. Resize/distribute options

### Phase 3: Menu Integration
1. Build menus from existing `KeyAssignment` enum
2. Use `CommandDef` system for menu metadata
3. Reuse menu item handler pattern from app.rs
4. Add `ArgType` filtering for context-aware menus

### Phase 4: Advanced Features
1. Recent panes / Recent connections
2. Pane arrangements / templates
3. Dynamic submenu population
4. Performance metrics / debug context menu

---

## 10. KEY ARCHITECTURAL DECISIONS

1. **Menu Item Actions**: Use existing `KeyAssignment` enum with `RepresentedItem` wrapper pattern
2. **Platform Abstraction**: `#[cfg(target_os = "macos")]` for native menus
3. **UI Item Tracking**: `UIItem` system enables hit-testing and interaction dispatch
4. **Pane State**: Binary tree in Tab allows flexible split operations
5. **Event Dispatch**: Route through `TermWindowNotif::PerformAssignment` to maintain consistency

---

## 11. TESTING CONSIDERATIONS

### Test Files to Create
- `tests/pane_context_menu.rs` - Test menu creation/display
- `tests/split_operations.rs` - Verify split through context menu works

### Existing Test Pattern
- `mux/src/tab.rs` (L:900+): `#[test] fn tab_splitting()` shows unit test pattern
- Integration via `wezterm-gui` mouse event tests

---

## CONCLUSION

The WezTerm codebase has:
1. ✅ Mature pane management with binary tree splits
2. ✅ Working macOS menu infrastructure with RepresentedItem pattern
3. ✅ Robust command/action system (KeyAssignment enum)
4. ✅ UI hit-testing framework for interaction detection
5. ✅ Clear separation of platform-specific code

**For context menu implementation**:
- Extend `mouse_event_impl()` to detect right-clicks on panes
- Create `build_pane_context_menu()` similar to `application_dock_menu()`
- Use `RepresentedItem::KeyAssignment` pattern for menu actions
- Dispatch through existing `TermWindowNotif::PerformAssignment` system
