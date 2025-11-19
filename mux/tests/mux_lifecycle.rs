// Integration tests for mux window/tab/pane lifecycle
use config::ConfigHandle;
use mux::domain::{Domain, LocalDomain};
use mux::{Mux, MuxNotification};
use portable_pty::CommandBuilder;
use std::sync::Arc;

fn setup_test_mux() -> Arc<Mux> {
    let _ = env_logger::builder().is_test(true).try_init();
    let mux = Mux::new(Some(Arc::new(ConfigHandle::default_config())));
    mux.register_domain(Arc::new(LocalDomain::new("local").unwrap()));
    mux
}

#[test]
fn test_window_creation() {
    let mux = setup_test_mux();

    // Verify initial state
    assert!(mux.is_empty());

    // Create a window
    let (window_id, _) = mux.new_empty_window(None, None);

    // Verify window was created
    assert!(!mux.is_empty());
    assert!(mux.get_window(window_id).is_some());

    let window = mux.get_window(window_id).unwrap();
    assert_eq!(window.window_id(), window_id);
}

#[test]
fn test_window_removal() {
    let mux = setup_test_mux();

    let (window_id, _) = mux.new_empty_window(None, None);
    assert!(mux.get_window(window_id).is_some());

    // Remove the window
    mux.remove_window(window_id);

    // Verify window was removed
    assert!(mux.get_window(window_id).is_none());
    assert!(mux.is_empty());
}

#[test]
fn test_multiple_windows() {
    let mux = setup_test_mux();

    // Create multiple windows
    let (window1, _) = mux.new_empty_window(None, None);
    let (window2, _) = mux.new_empty_window(None, None);
    let (window3, _) = mux.new_empty_window(None, None);

    // Verify all windows exist
    assert!(mux.get_window(window1).is_some());
    assert!(mux.get_window(window2).is_some());
    assert!(mux.get_window(window3).is_some());

    // Verify window IDs are unique
    assert_ne!(window1, window2);
    assert_ne!(window2, window3);
    assert_ne!(window1, window3);
}

#[test]
fn test_tab_creation_and_addition() {
    let mux = setup_test_mux();
    let (window_id, tab_id) = mux.new_empty_window(None, None);

    // Verify tab was created
    assert!(mux.get_tab(tab_id).is_some());

    let tab = mux.get_tab(tab_id).unwrap();
    assert_eq!(tab.tab_id(), tab_id);

    // Verify tab is in the window
    let window = mux.get_window(window_id).unwrap();
    let tabs = window.iter().map(|t| t.tab_id()).collect::<Vec<_>>();
    assert!(tabs.contains(&tab_id));
}

#[test]
fn test_tab_removal() {
    let mux = setup_test_mux();
    let (_window_id, tab_id) = mux.new_empty_window(None, None);

    assert!(mux.get_tab(tab_id).is_some());

    // Remove the tab
    mux.remove_tab(tab_id);

    // Verify tab was removed
    assert!(mux.get_tab(tab_id).is_none());
}

#[test]
fn test_workspace_assignment() {
    let mux = setup_test_mux();

    // Create window in default workspace
    let (window_id, _) = mux.new_empty_window(None, None);

    let window = mux.get_window(window_id).unwrap();
    assert_eq!(window.get_workspace(), mux::DEFAULT_WORKSPACE);

    // Create window in custom workspace
    let (window_id2, _) = mux.new_empty_window(Some("test-workspace"), None);
    let window2 = mux.get_window(window_id2).unwrap();
    assert_eq!(window2.get_workspace(), "test-workspace");
}

#[test]
fn test_get_pane_renders_ok() {
    let mux = setup_test_mux();
    let (_window_id, tab_id) = mux.new_empty_window(None, None);

    let tab = mux.get_tab(tab_id).unwrap();
    let panes = tab.iter_panes_ignoring_zoom();

    // Should have at least one pane (default pane)
    assert!(!panes.is_empty());
}

#[test]
fn test_notification_subscription() {
    let mux = setup_test_mux();

    // Subscribe to notifications
    let sub = mux.subscribe();

    // Create a window (should trigger notification)
    let (window_id, _) = mux.new_empty_window(None, None);

    // Check for WindowCreated notification
    let mut found_window_created = false;
    while let Ok(notif) = sub.try_recv() {
        if let MuxNotification::WindowCreated(wid) = notif {
            if wid == window_id {
                found_window_created = true;
                break;
            }
        }
    }

    assert!(found_window_created, "Should receive WindowCreated notification");
}

#[test]
fn test_mux_is_empty() {
    let mux = setup_test_mux();

    // Initially empty
    assert!(mux.is_empty());

    // Not empty after creating window
    let (window_id, _) = mux.new_empty_window(None, None);
    assert!(!mux.is_empty());

    // Empty again after removing window
    mux.remove_window(window_id);
    assert!(mux.is_empty());
}

#[test]
fn test_iter_windows() {
    let mux = setup_test_mux();

    // Create multiple windows
    let (w1, _) = mux.new_empty_window(None, None);
    let (w2, _) = mux.new_empty_window(None, None);
    let (w3, _) = mux.new_empty_window(None, None);

    // Collect window IDs
    let window_ids: Vec<_> = mux.iter_windows().map(|w| w.window_id()).collect();

    // Verify all windows are present
    assert_eq!(window_ids.len(), 3);
    assert!(window_ids.contains(&w1));
    assert!(window_ids.contains(&w2));
    assert!(window_ids.contains(&w3));
}

#[test]
fn test_get_active_tab_for_window() {
    let mux = setup_test_mux();
    let (window_id, tab_id) = mux.new_empty_window(None, None);

    // Get active tab for window
    let active_tab = mux.get_active_tab_for_window(window_id);
    assert!(active_tab.is_some());
    assert_eq!(active_tab.unwrap().tab_id(), tab_id);
}

#[test]
fn test_domain_registration() {
    let mux = setup_test_mux();

    // Get local domain
    let domain = mux.get_domain_by_name("local");
    assert!(domain.is_some());

    let domain = domain.unwrap();
    assert_eq!(domain.domain_name(), "local");
}

#[test]
fn test_window_workspace_operations() {
    let mux = setup_test_mux();

    // Create windows in different workspaces
    let (w1, _) = mux.new_empty_window(Some("workspace1"), None);
    let (w2, _) = mux.new_empty_window(Some("workspace2"), None);
    let (w3, _) = mux.new_empty_window(Some("workspace1"), None);

    // Get windows for workspace1
    let ws1_windows: Vec<_> = mux
        .iter_windows()
        .filter(|w| w.get_workspace() == "workspace1")
        .map(|w| w.window_id())
        .collect();

    assert_eq!(ws1_windows.len(), 2);
    assert!(ws1_windows.contains(&w1));
    assert!(ws1_windows.contains(&w3));

    // Get windows for workspace2
    let ws2_windows: Vec<_> = mux
        .iter_windows()
        .filter(|w| w.get_workspace() == "workspace2")
        .map(|w| w.window_id())
        .collect();

    assert_eq!(ws2_windows.len(), 1);
    assert!(ws2_windows.contains(&w2));
}
