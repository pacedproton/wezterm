use libvt::{create_terminal, KeyCode, KeyModifiers};

fn main() {
    // Create a 80x24 terminal
    let mut term = create_terminal(80, 24);

    // Write some text with escape sequences
    let text = b"Hello \x1b[1;32mWorld\x1b[0m!\r\n";
    term.write(text);

    // Test cursor positioning
    term.write(b"\x1b[10;20H"); // Move cursor to row 10, col 20
    term.write(b"Positioned text");

    // Test bell
    term.write(b"\x07");

    // Get screen content
    println!("=== Screen Content ===");
    for row in 0..11 {
        let line = term.get_line(row).unwrap();
        let text = line.to_string();
        if !text.trim().is_empty() {
            println!("Row {}: {}", row, text.trim());
        }
    }

    // Test key encoding
    println!("\n=== Key Encoding ===");
    let enter_bytes = term.key_down(KeyCode::Enter, KeyModifiers::empty());
    println!("Enter key generates: {:?}", enter_bytes);

    let ctrl_c = term.key_down(KeyCode::Char('c'), KeyModifiers::CONTROL);
    println!("Ctrl+C generates: {:?} (should be [3] = 0x03)", ctrl_c);

    let tab_key = term.key_down(KeyCode::Tab, KeyModifiers::empty());
    println!("Tab generates: {:?}", tab_key);

    let arrow_up = term.key_down(KeyCode::Up, KeyModifiers::empty());
    println!("Arrow Up generates: {:?}", arrow_up);

    // Check events
    let events = term.take_events();
    println!("\n=== Events Generated ===");
    for event in events {
        println!("Event: {:?}", event);
    }

    let (cols, rows) = term.size();
    println!("\n=== Terminal Info ===");
    println!("Terminal size: {}x{}", cols, rows);
    println!("Cursor position: {:?}", term.cursor_position());
    println!("Cursor visible: {:?}", term.cursor().visibility);
    println!("Cursor shape: {:?}", term.cursor().shape);
    println!("Terminal title: {:?}", term.title());
}
