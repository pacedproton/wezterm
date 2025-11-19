# WezTerm Context Menu Analysis - Documentation Index

This folder contains comprehensive documentation for implementing native macOS context menus for WezTerm panes and split dividers.

## Document Overview

### 1. **wezterm_pane_menu_findings.md** (19 KB) - MAIN FINDINGS DOCUMENT
**Start here!** Contains complete architectural analysis:
- Pane management structures (Tab, Pane, Split)
- macOS menu system implementation
- GUI pane rendering and interaction
- Command system and KeyAssignments
- Metal renderer overview
- Application event dispatch
- Integration points for context menus
- File references and quick lookup tables
- Implementation roadmap (Phase 1-4)
- Key architectural decisions
- Testing considerations

**Best for**: Understanding the overall architecture and finding specific components

**Key sections**:
- Section 1: Pane Management Architecture
- Section 2: macOS Menu Architecture
- Section 3: GUI Pane Rendering & Interaction
- Section 7: Integration Points for Context Menus
- Section 8: File References Quick Lookup
- Section 9: Implementation Roadmap

---

### 2. **wezterm_file_structure_map.txt** (12 KB) - VISUAL REFERENCE
Directory tree and navigation guide:
- Complete directory structure with annotations
- Key integration points with line numbers
- Critical data structures
- Code patterns to follow
- Specific lines of code to modify
- Relative file paths for quick reference

**Best for**: Finding specific files and understanding how they relate

**Key sections**:
- Directory Tree
- Key Integration Points
- Critical Data Structures
- Code Patterns to Follow
- Lines of Code to Modify/Extend
- Relative File Paths

---

### 3. **CONTEXT_MENU_IMPLEMENTATION_GUIDE.md** (12 KB) - QUICK START GUIDE
Implementation cookbook with copy-paste code:
- Three main steps to implement context menus
- Complete code examples
- Existing code patterns to follow
- Key data structures with type definitions
- List of files to modify
- Available KeyAssignments for menu items
- Testing instructions
- Objective-C interop reference
- Common issues and solutions
- Phased implementation plan

**Best for**: Actually implementing the feature; has working code examples

**Key sections**:
- Quick Start (3 main steps)
- Existing Code Patterns
- Files to Modify
- Context Menu Items - KeyAssignments
- Testing the Implementation
- Common Issues & Solutions

---

## Quick Navigation

### I need to understand...

#### The pane/tab/split architecture
→ Read `wezterm_pane_menu_findings.md` **Section 1**

#### The macOS menu system
→ Read `wezterm_pane_menu_findings.md` **Section 2**

#### Where mouse events are handled
→ Read `wezterm_file_structure_map.txt` **Integration Point 1**
→ Then `wezterm_pane_menu_findings.md` **Section 3.3**

#### Where split dividers are rendered
→ Read `wezterm_file_structure_map.txt` **Integration Point 2**
→ Then `wezterm_pane_menu_findings.md` **Section 3.2**

#### How menus currently work
→ Read `wezterm_pane_menu_findings.md` **Section 2**

#### The specific file paths and locations
→ Read `wezterm_file_structure_map.txt` **Directory Tree**
→ Then `wezterm_pane_menu_findings.md` **Section 8**

#### Code to copy/paste for implementation
→ Read `CONTEXT_MENU_IMPLEMENTATION_GUIDE.md` **Quick Start**

#### What KeyAssignments are available
→ Read `CONTEXT_MENU_IMPLEMENTATION_GUIDE.md` **Context Menu Items**

#### Common pitfalls and solutions
→ Read `CONTEXT_MENU_IMPLEMENTATION_GUIDE.md` **Common Issues & Solutions**

---

## Key Files to Modify (At a Glance)

1. **wezterm-gui/src/termwindow/mouseevent.rs** (line ~525)
   - Extend mouse_event_impl() to detect right-click
   - Currently handles ShowLauncher for new-tab-button
   - → Need to add pane context menu detection

2. **wezterm-gui/src/termwindow/mod.rs**
   - Add new method: show_pane_context_menu()
   - Add module: mod context_menu;
   - → Integration point for menu display

3. **wezterm-gui/src/termwindow/context_menu.rs** (NEW FILE)
   - Create build_pane_context_menu()
   - Create build_split_context_menu() (future)
   - → Contains menu building logic

4. **window/src/os/macos/menu.rs**
   - Already has Menu and MenuItem classes
   - → Ready to use as-is, no changes needed

5. **window/src/os/macos/app.rs**
   - Already has wezterm_perform_key_assignment handler
   - → Ready to use as-is, no changes needed

---

## Core Concepts Summary

### Pane Management
- **PaneId**: Unique identifier (usize) for each pane
- **Tab**: Container of panes, organized in binary tree
- **SplitDirection**: Horizontal or Vertical split
- **SplitRequest**: Parameters for creating a split
- **PositionedPane**: Pane with calculated position on screen
- **PositionedSplit**: Split divider with calculated position

### Menu System (macOS)
- **Menu**: Wraps NSMenu*
- **MenuItem**: Wraps NSMenuItem*
- **RepresentedItem**: Data attached to menu item (in this case, KeyAssignment)
- **Selector**: `sel!(weztermPerformKeyAssignment:)` - routes to handler in app.rs

### GUI Layer
- **UIItem**: Clickable/interactive UI element (tab, split, scrollbar, etc.)
- **UIItemType**: Type of UI element
- **Hit-testing**: Finding which UI element was clicked
- **MouseEvent**: Contains position and button information

### Event Dispatch
1. Right-click detected in mouse_event_impl()
2. Context menu created with menu items
3. Menu items have KeyAssignment attached via RepresentedItem
4. User selects menu item → selector calls wezterm_perform_key_assignment()
5. KeyAssignment is extracted and dispatched via TermWindowNotif::PerformAssignment

---

## Implementation Roadmap

### Phase 1: Basic Right-Click on Pane (CURRENT)
- [ ] Detect right-click on pane content area
- [ ] Create native NSMenu with basic options
- [ ] Display menu at mouse position
- [ ] Execute selected action via KeyAssignment

**Estimated effort**: 2-3 hours
**Files to touch**: 3 (mouseevent.rs, mod.rs, new context_menu.rs)

### Phase 2: Enhanced Context Menus
- [ ] Right-click on split dividers
- [ ] Right-click on tabs
- [ ] Pane movement options (move to new window/tab)
- [ ] Size adjustment options

**Estimated effort**: 2-3 hours
**Files to touch**: 2-3

### Phase 3: Menu Integration
- [ ] Build menus from CommandDef system
- [ ] Add keyboard shortcuts to menu items
- [ ] Argument filtering (ActivePane, ActiveTab, ActiveWindow)
- [ ] Submenu support for related operations

**Estimated effort**: 3-4 hours
**Files to touch**: 2-3

### Phase 4: Advanced Features
- [ ] Recent panes submenu
- [ ] Pane arrangements/templates
- [ ] Dynamic menu population
- [ ] Performance metrics/debug context menu

**Estimated effort**: 4-6 hours
**Files to touch**: 3-4

---

## Technical Deep Dives

### How Panes Are Organized
```
Tab
└── Tree (binary tree of panes)
    ├── Node (split)
    │   ├── Left leaf: Pane A
    │   └── Right leaf: Pane B
    └── Active: Pane A (index 0)
```

When you right-click on a pane, you get its PaneId from the active pane.

### How Splits Are Detected
```
UI Items (rendered last frame)
└── UIItem::Split (PositionedSplit)
    ├── Coordinates (x, y, width, height)
    └── Type: Horizontal or Vertical

Right-click detection:
1. Get mouse coordinates
2. Hit-test against UIItem::Split
3. If hit, show split context menu
```

### How Menu Actions Execute
```
Menu Item (NSMenuItem)
└── Represented Object: RepresentedItem::KeyAssignment(SplitPane(...))
    └── When clicked:
        1. wezterm_perform_key_assignment() called
        2. Extracts KeyAssignment from represented object
        3. Routes to ApplicationEvent::PerformKeyAssignment()
        4. Dispatched through TermWindowNotif::PerformAssignment
        5. Handler executes the action (e.g., split_pane())
```

---

## Code Quality Checklist

When implementing context menus:

- [ ] Use existing macOS menu abstractions (Menu, MenuItem, RepresentedItem)
- [ ] Follow existing code patterns (dock menu, app delegate)
- [ ] Wrap macOS-specific code with `#[cfg(target_os = "macos")]`
- [ ] Use existing KeyAssignment enum values
- [ ] Return early from mouse_event_impl after showing menu
- [ ] Log debug messages for troubleshooting
- [ ] Test with multiple pane configurations (zoomed, nested splits, etc.)
- [ ] Verify menu items execute correct actions
- [ ] Check that menu appears at correct coordinates

---

## References

### Files in This Analysis
1. **wezterm_pane_menu_findings.md** - Full architectural analysis
2. **wezterm_file_structure_map.txt** - Visual file structure and integration points
3. **CONTEXT_MENU_IMPLEMENTATION_GUIDE.md** - Implementation cookbook with examples

### Key Source Files in WezTerm
- `mux/src/pane.rs` - Pane trait
- `mux/src/tab.rs` - Tab and split structures
- `window/src/os/macos/menu.rs` - Menu system
- `window/src/os/macos/app.rs` - App delegate and menu handlers
- `wezterm-gui/src/termwindow/mouseevent.rs` - Mouse event handling
- `wezterm-gui/src/termwindow/render/split.rs` - Split rendering
- `wezterm-gui/src/commands.rs` - Command definitions and menu bar

### External Resources
- macOS menu documentation: NSMenu, NSMenuItem
- Cocoa bindings in Rust: cocoa crate
- Objective-C interop: objc crate

---

## Getting Started

### Step 1: Read the Architecture
1. Start with `wezterm_pane_menu_findings.md` Section 1 (Pane Management)
2. Then read Section 2 (macOS Menu Architecture)
3. Skip to Section 7 (Integration Points) for context menu specifics

### Step 2: Locate the Code
1. Use `wezterm_file_structure_map.txt` to find exact file paths
2. Open each key file in your editor
3. Read the existing code patterns

### Step 3: Plan Implementation
1. Follow the 3-step outline in `CONTEXT_MENU_IMPLEMENTATION_GUIDE.md`
2. Create context_menu.rs module first
3. Extend mouse_event_impl() next
4. Test incrementally

### Step 4: Build and Test
```bash
cargo build --package wezterm-gui
./target/debug/wezterm
# Right-click on a pane to test
```

---

## Questions?

Refer to the specific document:
- **"What does X do?"** → wezterm_pane_menu_findings.md
- **"Where is X located?"** → wezterm_file_structure_map.txt
- **"How do I implement X?"** → CONTEXT_MENU_IMPLEMENTATION_GUIDE.md
- **"What code should I write?"** → CONTEXT_MENU_IMPLEMENTATION_GUIDE.md (look for code blocks)

---

Generated: November 19, 2025
Analysis conducted on WezTerm branch: `claude/wezterm-mac-native-01LfwYNrAg32v2168oHT7fkf`
Analyzed codebase: `/home/user/wezterm/`
