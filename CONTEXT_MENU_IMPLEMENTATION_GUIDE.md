# WezTerm Context Menu Implementation Guide

Quick reference for implementing native macOS context menus for panes and split dividers.

## Quick Start - Three Main Steps

### Step 1: Detect Right-Click on Pane
**File**: `wezterm-gui/src/termwindow/mouseevent.rs` (around line 520)

Current code at `do_new_tab_button_click()`:
```rust
fn do_new_tab_button_click(&mut self, button: MousePress) {
    let action = match button {
        MousePress::Left => Some(KeyAssignment::SpawnTab(...)),
        MousePress::Right => Some(KeyAssignment::ShowLauncher),  // Current behavior
        MousePress::Middle => None,
    };
}
```

**What to add** (pseudocode):
```rust
pub fn mouse_event_impl(&mut self, event: MouseEvent, context: &dyn WindowOps) {
    // ... existing code ...
    
    // Check for right-click on pane content area
    if event.kind == WMEK::Press(MousePress::Right) {
        if let Some(pane) = self.get_active_pane_or_overlay() {
            // Show pane context menu at mouse coordinates
            self.show_pane_context_menu(pane.pane_id(), event.coords.x, event.coords.y);
            return;
        }
    }
}
```

### Step 2: Build Context Menu Using Native NSMenu
**File**: Create `wezterm-gui/src/termwindow/context_menu.rs` (new file)

```rust
use window::os::macos::menu::{Menu, MenuItem, RepresentedItem};
use config::keyassignment::KeyAssignment;
use objc::*;

#[cfg(target_os = "macos")]
pub fn build_pane_context_menu(pane_id: mux::pane::PaneId) -> Menu {
    let menu = Menu::new_with_title("");
    
    // Split Horizontally
    let split_h = MenuItem::new_with(
        "Split Horizontally",
        Some(sel!(weztermPerformKeyAssignment:)),
        ""
    );
    split_h.set_represented_item(
        RepresentedItem::KeyAssignment(
            KeyAssignment::SplitPane(config::keyassignment::SplitSize::Percent(50))
        )
    );
    menu.add_item(&split_h);
    
    // Split Vertically
    let split_v = MenuItem::new_with(
        "Split Vertically",
        Some(sel!(weztermPerformKeyAssignment:)),
        ""
    );
    split_v.set_represented_item(
        RepresentedItem::KeyAssignment(
            KeyAssignment::SplitPane(config::keyassignment::SplitSize::Percent(50))
        )
    );
    menu.add_item(&split_v);
    
    // Separator
    menu.add_item(&MenuItem::new_separator());
    
    // Close Pane
    let close = MenuItem::new_with(
        "Close Pane",
        Some(sel!(weztermPerformKeyAssignment:)),
        ""
    );
    close.set_represented_item(
        RepresentedItem::KeyAssignment(KeyAssignment::CloseCurrentPane)
    );
    menu.add_item(&close);
    
    // Zoom
    let zoom = MenuItem::new_with(
        "Zoom Pane",
        Some(sel!(weztermPerformKeyAssignment:)),
        ""
    );
    zoom.set_represented_item(
        RepresentedItem::KeyAssignment(KeyAssignment::TogglePaneZoomState)
    );
    menu.add_item(&zoom);
    
    menu
}

#[cfg(not(target_os = "macos"))]
pub fn build_pane_context_menu(_pane_id: mux::pane::PaneId) -> Menu {
    // Return empty menu on non-macOS platforms
    Menu::new_with_title("")
}
```

### Step 3: Display Menu at Mouse Position
**File**: `wezterm-gui/src/termwindow/mod.rs` (add to TermWindow impl)

```rust
impl TermWindow {
    #[cfg(target_os = "macos")]
    fn show_pane_context_menu(
        &mut self,
        pane_id: PaneId,
        x: isize,
        y: isize,
    ) {
        use window::os::macos::menu::Menu;
        use objc::{msg_send, *};
        use cocoa::appkit::NSApplication;
        
        let menu = crate::termwindow::context_menu::build_pane_context_menu(pane_id);
        
        unsafe {
            // Convert pixel coordinates to window coordinates
            let app = NSApplication::sharedApplication(nil);
            let event: id = msg_send![app, currentEvent];
            
            // Display menu at cursor location
            let () = msg_send![*menu.menu, popUpMenuPositioningItem:nil atLocation:cocoa::foundation::NSPoint::new(x as f64, y as f64) inView:nil];
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    fn show_pane_context_menu(&mut self, _pane_id: PaneId, _x: isize, _y: isize) {
        // No-op on non-macOS platforms
    }
}
```

---

## Existing Code Patterns to Follow

### Pattern A: Menu Item Creation (from app.rs:137)
```rust
let item = MenuItem::new_with(
    "New Window",
    Some(sel!(weztermPerformKeyAssignment:)),
    ""
);
item.set_represented_item(
    RepresentedItem::KeyAssignment(KeyAssignment::SpawnWindow)
);
dock_menu.add_item(&item);
```

### Pattern B: Hit-Testing (from mouseevent.rs:27)
```rust
fn resolve_ui_item(&self, event: &MouseEvent) -> Option<UIItem> {
    let x = event.coords.x;
    let y = event.coords.y;
    self.ui_items
        .iter()
        .rev()  // Top-to-bottom z-order
        .find(|item| item.hit_test(x, y))
        .cloned()
}
```

### Pattern C: Event Dispatch (from mouseevent.rs:440)
```rust
window.notify(TermWindowNotif::PerformAssignment {
    pane_id: pane.pane_id(),
    assignment: action,
    tx: None,
});
```

---

## Key Data Structures

### Identifying a Pane
```rust
type PaneId = usize;  // Unique identifier for a pane
// Get active pane: self.get_active_pane() -> Option<Arc<dyn Pane>>
// Get pane ID: pane.pane_id() -> PaneId
```

### Mouse Event
```rust
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub coords: Point,  // x, y coordinates in pixels
    // ... other fields
}

pub enum MouseEventKind {
    Press(MousePress),
    Release(MousePress),
    Move,
    VertWheel(i32),
    HorzWheel(i32),
}

pub enum MousePress {
    Left,
    Right,
    Middle,
}
```

### Split Information (from split.rs hit-test)
```rust
pub struct PositionedSplit {
    pub index: usize,              // Node index in binary tree
    pub direction: SplitDirection, // Horizontal or Vertical
    pub left: usize,               // Cell coordinate
    pub top: usize,                // Cell coordinate
    pub size: usize,               // Width (vertical) or height (horizontal) in cells
}

pub enum SplitDirection {
    Horizontal,
    Vertical,
}
```

---

## Files to Modify

### 1. `wezterm-gui/src/termwindow/mouseevent.rs`
- Extend `mouse_event_impl()` to detect right-click on panes
- Call new `show_pane_context_menu()` method

**Lines**: Around 520-530 (in `do_new_tab_button_click()`)

### 2. `wezterm-gui/src/termwindow/mod.rs`
- Add `show_pane_context_menu()` method to TermWindow impl
- Add `use` statements for context_menu module

**Lines**: Add near other mouse event handlers

### 3. Create `wezterm-gui/src/termwindow/context_menu.rs` (NEW)
- Implement `build_pane_context_menu()`
- Implement `build_split_context_menu()` (future)

### 4. `wezterm-gui/src/termwindow/mod.rs` (module declaration)
- Add `mod context_menu;` or `pub mod context_menu;`

---

## Context Menu Items - KeyAssignments to Use

### Pane Operations
```rust
KeyAssignment::SplitPane(SplitSize)           // Split with size
KeyAssignment::SplitHorizontally              // Split 50/50 horizontal
KeyAssignment::SplitVertically                // Split 50/50 vertical
KeyAssignment::CloseCurrentPane               // Close the pane
KeyAssignment::TogglePaneZoomState            // Zoom/unzoom
KeyAssignment::RotatePanes                    // Rotate panes in tab
KeyAssignment::MoveTabToNewWindow             // Move to new window
KeyAssignment::AdjustPaneSize(dir, amount)    // Resize pane
KeyAssignment::ActivatePaneDirection(dir)     // Focus adjacent pane
```

### Navigation
```rust
KeyAssignment::ActivateTabRelative(delta)     // Switch to another tab
KeyAssignment::ActivatePaneDirection(dir)     // Focus neighboring pane
```

### Tab Operations (for tab bar context menu)
```rust
KeyAssignment::CloseCurrentTab
KeyAssignment::RenameWorkspace(name)
KeyAssignment::MoveTabToNewWindow
```

---

## Testing the Implementation

### 1. Build with debug info
```bash
cargo build --package wezterm-gui 2>&1 | tee build.log
```

### 2. Test with target
```bash
./target/debug/wezterm
```

### 3. Right-click on pane
- Should show context menu
- Menu items should execute their assigned KeyAssignments
- Menu should disappear after selection or clicking elsewhere

### 4. Verify split dividers
```rust
// In mouseevent.rs: Check that UIItem::Split is being created
log::debug!("Resolved UI item: {:?}", resolved_item);
```

---

## Objective-C Interop Reference

### Selector (SEL) for Menu Action
```rust
Some(sel!(weztermPerformKeyAssignment:))
```

This selector routes to the handler in `window/src/os/macos/app.rs:96`:
```rust
extern "C" fn wezterm_perform_key_assignment(
    _self: &mut Object,
    _sel: Sel,
    menu_item: *mut Object,
)
```

### Converting Pixel to Window Coordinates
```rust
unsafe {
    let app = NSApplication::sharedApplication(nil);
    // The coordinates in MouseEvent are already in window space
    // You can use them directly for menu display
}
```

### Showing NSMenu Programmatically
```rust
unsafe {
    let () = msg_send![*menu.menu, 
        popUpMenuPositioningItem:nil 
        atLocation:NSPoint::new(x as f64, y as f64) 
        inView:nil];
}
```

---

## Common Issues & Solutions

### Issue: Menu doesn't appear
**Solution**: Check that `show_pane_context_menu()` is actually being called.
```rust
log::debug!("Showing context menu at ({}, {})", x, y);
```

### Issue: Menu item selector not found
**Solution**: Make sure `sel!(weztermPerformKeyAssignment:)` is declared with proper imports.
```rust
use objc::*;  // Provides sel!() macro
```

### Issue: KeyAssignment not recognized
**Solution**: Check that the KeyAssignment enum variant exists in `config/src/keyassignment.rs`.
Use existing variants found in `wezterm-gui/src/commands.rs`.

### Issue: Right-click goes to pane instead of menu
**Solution**: Return early from mouse event handler after showing menu:
```rust
if event.kind == WMEK::Press(MousePress::Right) {
    self.show_pane_context_menu(...);
    return;  // Don't pass to pane
}
```

---

## Next Steps

1. **Phase 1**: Implement basic right-click on pane
   - [ ] Create context_menu.rs module
   - [ ] Add show_pane_context_menu() to TermWindow
   - [ ] Extend mouse_event_impl() to detect right-click
   - [ ] Test with 3-4 basic menu items

2. **Phase 2**: Add split divider context menu
   - [ ] Detect right-click on UIItemType::Split
   - [ ] Build split-specific menu (Swap, Rotate, Equalize)
   - [ ] Test menu interaction

3. **Phase 3**: Add tab bar context menu
   - [ ] Detect right-click on tab items
   - [ ] Add tab-specific options (Close, Rename, Move)
   - [ ] Integrate with existing tab bar code

4. **Phase 4**: Advanced features
   - [ ] Dynamic menu generation from CommandDef
   - [ ] Keyboard shortcuts in menu items
   - [ ] Recent panes submenu
   - [ ] Performance/debug submenu

---

## References

### Cocoa/Objective-C
- NSMenu: Create menus
- NSMenuItem: Menu items with actions
- NSApplication: Get shared app instance
- msg_send!: Call Objective-C methods from Rust

### WezTerm
- MouseEvent: `window::MouseEvent`
- KeyAssignment: `config::keyassignment::KeyAssignment`
- UIItem: `wezterm_gui::termwindow::UIItem`
- Pane: `mux::pane::Pane` trait

### Related Files
- Menu system: `window/src/os/macos/menu.rs`
- App delegate: `window/src/os/macos/app.rs`
- Mouse events: `wezterm-gui/src/termwindow/mouseevent.rs`
- Split rendering: `wezterm-gui/src/termwindow/render/split.rs`
