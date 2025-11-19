# Debugging WezTerm

This guide provides comprehensive information on debugging WezTerm, including logging, tracing, common issues, and debugging techniques.

## Table of Contents

1. [Enabling Logging](#enabling-logging)
2. [Log Levels and Filtering](#log-levels-and-filtering)
3. [Debug Builds](#debug-builds)
4. [Debugging Specific Components](#debugging-specific-components)
5. [Common Debugging Scenarios](#common-debugging-scenarios)
6. [Using GDB/LLDB](#using-gdblldb)
7. [Performance Profiling](#performance-profiling)
8. [Memory Debugging](#memory-debugging)
9. [Test Coverage and Instrumentation](#test-coverage-and-instrumentation)

---

## Enabling Logging

WezTerm uses the `log` crate for logging throughout the codebase. Enable logging by setting the `WEZTERM_LOG` environment variable.

### Basic Logging

```bash
# Enable all debug logs
WEZTERM_LOG=debug wezterm

# Enable trace-level logging (very verbose)
WEZTERM_LOG=trace wezterm

# Enable info-level logging (default for most purposes)
WEZTERM_LOG=info wezterm
```

### Redirecting Logs to File

```bash
# Redirect logs to a file
WEZTERM_LOG=debug wezterm 2> /tmp/wezterm.log

# Or use wezterm's built-in log file location
# Logs are written to:
# - Linux/macOS: ~/.local/share/wezterm/
# - Windows: %APPDATA%\wezterm\
```

---

## Log Levels and Filtering

WezTerm supports fine-grained log filtering using the `env_logger` syntax.

### Available Log Levels

- `error` - Only errors
- `warn` - Warnings and errors
- `info` - Informational messages
- `debug` - Debug messages (recommended for development)
- `trace` - Very verbose tracing (can be overwhelming)

### Module-Specific Filtering

Enable logging for specific modules:

```bash
# Only log debug messages from the mux module
WEZTERM_LOG=mux=debug wezterm

# Log debug from mux, trace from wezterm_client
WEZTERM_LOG=mux=debug,wezterm_client=trace wezterm

# Log debug from everything except noise from one module
WEZTERM_LOG=debug,overly_chatty_module=info wezterm

# Complex filter: debug for most, trace for specific subsystems
WEZTERM_LOG=debug,mux=trace,wezterm_client=trace,wezterm_mux_server_impl=trace wezterm
```

### Key Modules to Debug

| Module | Purpose | When to Debug |
|--------|---------|---------------|
| `mux` | Multiplexer (Window/Tab/Pane lifecycle) | Issues with window/tab/pane management |
| `wezterm_client` | Client-side RPC communication | Connection issues, remote multiplexer problems |
| `wezterm_mux_server_impl` | Server-side RPC handling | Server connection issues |
| `wezterm_gui` | GUI rendering and event handling | Rendering issues, input problems |
| `wezterm_font` | Font loading and rendering | Font-related issues |
| `config` | Configuration parsing | Config file issues |
| `termwiz` | Terminal UI library | Terminal emulation problems |
| `wezterm_term` | Terminal state machine | Escape sequence handling issues |

---

## Debug Builds

Debug builds include additional assertions and better backtraces.

### Building for Debugging

```bash
# Build without optimizations (faster compile, slower runtime)
cargo build

# Build with debug symbols in release mode (for profiling)
cargo build --release --profile release

# Run debug build directly
cargo run

# Run with verbose backtraces
RUST_BACKTRACE=1 cargo run
RUST_BACKTRACE=full cargo run  # Even more detailed
```

### Debug vs Release Performance

Debug builds are significantly slower but provide:
- Better error messages
- More detailed stack traces
- Additional runtime checks
- Easier step-through debugging

**Recommendation**: Use debug builds for development, release builds for testing performance issues.

---

## Debugging Specific Components

### Multiplexer (Mux) Debugging

The mux manages windows, tabs, and panes. Key debugging points:

```bash
# Enable mux debug logging
WEZTERM_LOG=mux=debug wezterm

# Common mux operations to trace:
# - Window creation: logs "new_empty_window: creating window_id=..."
# - Tab addition: logs "add_tab_to_window: adding tab_id=..."
# - Pane removal: logs "remove_pane: removing pane_id=..."
# - Tab removal: logs "remove_tab: removing tab_id=..."
```

**Key Functions (instrumented with logging)**:
- `Mux::new_empty_window()` - Window creation
- `Mux::add_tab_to_window()` - Tab lifecycle
- `Mux::remove_pane()` - Pane cleanup
- `Mux::remove_tab()` - Tab cleanup
- `Mux::prune_dead_windows()` - Window garbage collection

### Client-Server Debugging

For remote multiplexer debugging:

```bash
# Server-side logging
WEZTERM_LOG=wezterm_mux_server_impl=trace,mux=debug wezterm-mux-server start

# Client-side logging
WEZTERM_LOG=wezterm_client=trace,codec=debug wezterm connect unix://...
```

**What to look for**:
- Connection establishment messages
- Protocol version negotiation
- RPC call timing (via metrics)
- Serialization/deserialization errors

### Terminal Emulation Debugging

For escape sequence and terminal state issues:

```bash
# Enable terminal emulation debug logs
WEZTERM_LOG=wezterm_term=debug,vtparse=trace wezterm

# Record terminal sequences
script -c "WEZTERM_LOG=wezterm_term=trace wezterm" /tmp/terminal-session.log
```

### Font Rendering Debugging

```bash
# Debug font selection and rendering
WEZTERM_LOG=wezterm_font=debug,wezterm_gui::glyphcache=debug wezterm
```

**Common issues**:
- Font fallback chains
- Glyph cache misses
- HarfBuzz shaping problems
- Emoji rendering

### Configuration Debugging

```bash
# Debug Lua configuration loading
WEZTERM_LOG=config=debug wezterm

# Validate config without starting GUI
wezterm show-config

# Test specific config values
wezterm cli get-config
```

---

## Common Debugging Scenarios

### Scenario 1: WezTerm Crashes on Startup

```bash
# Get a backtrace
RUST_BACKTRACE=full WEZTERM_LOG=debug wezterm 2>&1 | tee crash.log

# If it crashes before logging works, use gdb:
gdb --args wezterm
(gdb) run
(gdb) bt  # backtrace after crash
```

### Scenario 2: High CPU Usage

```bash
# Enable performance metrics
WEZTERM_LOG=debug wezterm

# Check metrics output for:
# - RPC call timing: histogram!("rpc", ...)
# - Frame rendering time
# - Event processing time

# Profile with perf (Linux)
perf record -g wezterm
perf report
```

### Scenario 3: Memory Leak

```bash
# Build with dhat-heap feature
cargo build --features dhat-heap

# Run and analyze heap usage
WEZTERM_LOG=debug ./target/debug/wezterm-gui

# Memory profiling with valgrind (Linux)
valgrind --leak-check=full --show-leak-kinds=all wezterm
```

### Scenario 4: Rendering Glitches

```bash
# Debug GPU rendering
WEZTERM_LOG=wezterm_gui::renderstate=debug,wezterm_gui::glyphcache=trace wezterm

# Force software rendering (bypass GPU)
LIBGL_ALWAYS_SOFTWARE=1 wezterm  # Linux
# or
wezterm start -- --front-end Software  # All platforms

# Check shader compilation
WEZTERM_LOG=wezterm_gui=debug wezterm 2>&1 | grep -i shader
```

### Scenario 5: SSH/Remote Connection Issues

```bash
# Debug SSH connection
WEZTERM_LOG=wezterm_ssh=trace,wezterm_client=debug wezterm ssh hostname

# Debug mux server discovery
WEZTERM_LOG=wezterm_client::discovery=trace wezterm connect unix://...
```

### Scenario 6: Configuration Not Applied

```bash
# Check config loading
WEZTERM_LOG=config=debug wezterm

# Verify config location
wezterm show-config

# Test config syntax
wezterm cli get-config 2>&1 | grep -i error
```

---

## Using GDB/LLDB

### GDB Setup (Linux)

```bash
# Build with debug symbols
cargo build

# Start GDB
gdb ./target/debug/wezterm-gui

# Set breakpoints
(gdb) break main
(gdb) break mux::Mux::new_empty_window
(gdb) break 'wezterm_gui::termwindow::TermWindow::paint'

# Run
(gdb) run

# Common commands
(gdb) backtrace      # Show call stack
(gdb) frame N        # Switch to frame N
(gdb) print var      # Print variable
(gdb) info locals    # Show local variables
(gdb) continue       # Continue execution
(gdb) next           # Step over
(gdb) step           # Step into
```

### LLDB Setup (macOS/Linux)

```bash
# Build with debug symbols
cargo build

# Start LLDB
lldb ./target/debug/wezterm-gui

# Set breakpoints
(lldb) breakpoint set --name main
(lldb) breakpoint set --name mux::Mux::new_empty_window

# Run
(lldb) run

# Common commands
(lldb) bt            # Backtrace
(lldb) frame select N  # Switch frame
(lldb) p var         # Print variable
(lldb) continue      # Continue
(lldb) next          # Step over
(lldb) step          # Step into
```

### Debugging Rust with GDB/LLDB

Rust symbols can be long. Use these helpers:

```bash
# Install rust-gdb wrapper (better pretty-printing)
rust-gdb ./target/debug/wezterm-gui

# Install rust-lldb wrapper
rust-lldb ./target/debug/wezterm-gui
```

---

## Performance Profiling

### Using `perf` (Linux)

```bash
# Record performance profile
perf record -g --call-graph dwarf ./target/release/wezterm-gui

# View report
perf report

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg
```

### Using Instruments (macOS)

```bash
# Build release with debug symbols
cargo build --release

# Open in Instruments
instruments -t "Time Profiler" ./target/release/wezterm-gui
```

### Using Built-in Metrics

WezTerm includes metrics instrumentation:

```bash
# Check RPC timing
WEZTERM_LOG=debug wezterm 2>&1 | grep "rpc.method"

# Monitor in code:
# - histogram!("rpc", "method" => "method_name")
# - counter!("rpc.count", "method" => "method_name")
```

---

## Memory Debugging

### Heap Profiling with dhat

```bash
# Build with dhat feature
cargo build --features dhat-heap

# Run and generate heap profile
./target/debug/wezterm-gui

# Profile data written to dhat-heap.json
# View with https://nnethercote.github.io/dh_view/dh_view.html
```

### AddressSanitizer (ASan)

```bash
# Build with ASan (nightly Rust required)
RUSTFLAGS="-Z sanitizer=address" cargo +nightly build --target x86_64-unknown-linux-gnu

# Run
./target/x86_64-unknown-linux-gnu/debug/wezterm-gui
```

### Memory Leak Detection

```bash
# Valgrind (Linux)
valgrind --leak-check=full --show-leak-kinds=all ./target/debug/wezterm-gui

# Look for "definitely lost" and "possibly lost" blocks
```

---

## Test Coverage and Instrumentation

### Running Tests with Coverage

```bash
# Install tarpaulin (Linux only)
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir ./coverage

# View coverage/index.html in browser
```

### Running Tests with Logging

```bash
# Run tests with debug output
RUST_LOG=debug cargo test -- --nocapture

# Run specific test
cargo test test_window_creation -- --nocapture

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test
```

### Using nextest (Faster Test Runner)

```bash
# Install nextest
cargo install cargo-nextest

# Run all tests
cargo nextest run

# Run specific crate tests
cargo nextest run -p mux

# Run with retries for flaky tests
cargo nextest run --retries 3
```

---

## Debug Environment Variables Reference

| Variable | Values | Purpose |
|----------|--------|---------|
| `WEZTERM_LOG` | `error`, `warn`, `info`, `debug`, `trace` | Set log level |
| `RUST_BACKTRACE` | `0`, `1`, `full` | Enable backtraces |
| `RUST_LOG` | Same as `WEZTERM_LOG` | Alternative log variable |
| `LIBGL_ALWAYS_SOFTWARE` | `1` | Force software rendering (Linux) |
| `WEZTERM_LOG_FILE` | Path | Redirect logs to file |

---

## Getting Help

If you're stuck:

1. **Check logs**: `WEZTERM_LOG=debug wezterm 2> wezterm.log`
2. **Search issues**: https://github.com/wezterm/wezterm/issues
3. **Ask in Discussions**: https://github.com/wezterm/wezterm/discussions
4. **Join Matrix**: https://app.element.io/#/room/#wezterm:matrix.org

When reporting issues, include:
- WezTerm version (`wezterm --version`)
- OS and version
- Configuration file (sanitize sensitive data)
- **Full logs** with `WEZTERM_LOG=debug`
- Minimal reproduction steps

---

## Advanced Debugging Tips

### 1. Bisecting Regressions

```bash
# Use git bisect to find when a bug was introduced
git bisect start
git bisect bad HEAD
git bisect good v20230101-120000-abc123

# Test each commit:
cargo build && cargo test
git bisect good  # or git bisect bad

git bisect reset  # when done
```

### 2. Debugging Lua Configuration

```lua
-- Add to wezterm.lua for debugging
local wezterm = require 'wezterm'
wezterm.log_info("Config value: " .. tostring(some_variable))
wezterm.log_error("This shouldn't happen!")

return {
  -- your config
}
```

### 3. Debug GUI Event Loop

```bash
# Trace window events
WEZTERM_LOG=wezterm_gui::termwindow=trace wezterm 2>&1 | grep -i event
```

### 4. Test Specific Escape Sequences

```bash
# Test specific escape sequences
echo -e "\e[1;32mGreen Bold Text\e[0m" | WEZTERM_LOG=wezterm_term=debug wezterm
```

---

## Contributing Debug Improvements

When adding new features:

1. **Add logging**: Use `log::debug!()`, `log::trace!()`, etc.
2. **Add metrics**: Use `metrics::histogram!()` for timing
3. **Add tests**: Cover critical paths (see [test coverage analysis](/tmp/EXECUTIVE_SUMMARY.txt))
4. **Document debug flags**: Update this guide

---

**Last Updated**: 2025-11-19
**Maintainer**: WezTerm Team

For the latest debugging information, see: https://wezterm.org/troubleshooting.html
