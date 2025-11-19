# Debugging & Instrumentation Guide

Comprehensive debugging and profiling tools for the WezTerm Metal renderer.

## Quick Start

### Enable Debug Mode

```rust
use wezterm_metal::{MetalRenderer, RenderConfig, DebugConfig};

let debug_config = DebugConfig {
    enable_profiling: true,
    enable_gpu_validation: true,
    enable_overlay: true,
    enable_memory_tracking: true,
    ..Default::default()
};

let render_config = RenderConfig::default();
let renderer = MetalRenderer::new_with_debug(render_config, debug_config)?;
```

### View Performance Stats

```rust
// Get real-time statistics
let stats = renderer.profiler().get_stats();
stats.print();

// Example output:
// ╔═══════════════════════════════════════════╗
// ║     Performance Statistics                ║
// ╠═══════════════════════════════════════════╣
// ║ Total Frames:     1234                    ║
// ║ Dropped Frames:   12                      ║
// ║ Drop Rate:        0.97%                   ║
// ╠═══════════════════════════════════════════╣
// ║ Average FPS:      118.5                   ║
// ║ Avg Frame Time:   8.43ms                  ║
// ║ P95 Frame Time:   12.1ms                  ║
// ╚═══════════════════════════════════════════╝
```

## Performance Profiling

### Frame-Level Profiling

```rust
// Automatic profiling with RAII guard
let _frame = renderer.profiler().begin_frame();

// Your rendering code here...
renderer.render(&buffer, &drawable)?;

// Frame stats automatically recorded when guard drops
```

### Section-Level Profiling

```rust
use wezterm_metal::debug::TimingType;

{
    let _section = renderer.profiler().record_section_guard(
        "cell_processing",
        TimingType::Cpu
    );

    // Your code here
    process_cells(&buffer);
}

// Duration automatically recorded
```

### Using Macros

```rust
use wezterm_metal::{profile_section, profile_function};

fn my_rendering_function(profiler: &PerformanceProfiler) {
    profile_function!(profiler, {
        // Function code here
    });
}

fn specific_section(profiler: &PerformanceProfiler) {
    profile_section!(profiler, "glyph_upload", TimingType::Gpu, {
        upload_glyphs_to_gpu();
    });
}
```

### Performance Metrics

```rust
let stats = profiler.get_stats();

println!("FPS: {:.1}", stats.avg_fps);
println!("Frame time: {:.2}ms", stats.avg_frame_time_ms);
println!("P95 frame time: {:.2}ms", stats.p95_frame_time_ms);
println!("P99 frame time: {:.2}ms", stats.p99_frame_time_ms);
println!("Dropped frames: {} ({:.1}%)",
         stats.dropped_frames,
         (stats.dropped_frames as f32 / stats.total_frames as f32) * 100.0);
```

## GPU Debugging

### Enable Metal Validation

```bash
# Set environment variable before running
export METAL_DEVICE_WRAPPER_TYPE=1

# Run your application
cargo run --release
```

The validation layer will catch:
- API misuse
- Shader errors
- Resource violations
- Memory corruption
- Invalid state transitions

### Frame Capture

```rust
use wezterm_metal::GpuDebugger;

let gpu_debugger = GpuDebugger::new(&device, &debug_config);

// Capture a single frame
gpu_debugger.begin_capture();
renderer.render(&buffer, &drawable)?;
gpu_debugger.end_capture();

// Capture to file
gpu_debugger.start_programmatic_capture(&device)?;
// ... render frames ...
gpu_debugger.stop_programmatic_capture();
// View /tmp/wezterm_capture.gputrace in Xcode
```

### Viewing Captures in Xcode

1. Run with frame capture enabled
2. Open Xcode
3. Go to: **Window > Devices and Simulators > Open GPU Frame Debugger**
4. Load the `.gputrace` file from `/tmp/wezterm_capture.gputrace`

You'll see:
- Draw call hierarchy
- Shader performance
- Texture contents
- Buffer data
- GPU timeline

## Visual Debug Overlay

### Enable Overlay

```rust
use wezterm_metal::DebugOverlay;

let mut overlay = DebugOverlay::default();
overlay.show_fps = true;
overlay.show_frame_graph = true;
overlay.show_memory = true;
overlay.show_gpu = true;
overlay.opacity = 0.8;

// Generate overlay text
let text = overlay.render_to_string(&stats, current_frame);
println!("{}", text);
```

### Example Overlay Output

```
FPS: 118.5
Frame: 8.43ms
✨ FAST

Frame times: ▂▃▄▃▂▃▄▅▄▃▂▃▄▃▂▃▄▃▂▃

GPU:
  Time: 1.23ms
  Draws: 1
  Glyphs: 2048

Memory:
  Heap: 85.3MB
  GPU: 12.5MB
  Atlas: 4.2MB
```

### Frame Graph

```rust
use wezterm_metal::FrameGraph;

let mut graph = FrameGraph::new(120, 800, 200);

// Add samples each frame
graph.add_sample(frame_time_ms);

// Generate ASCII visualization
println!("{}", graph.to_ascii());
```

## Memory Tracking

### Enable Memory Tracking

```rust
use wezterm_metal::{MemoryTracker, AllocationType};

let tracker = MemoryTracker::new(true);

// Track allocations
let id = tracker.allocate(
    1024 * 1024,  // 1MB
    AllocationType::GpuTexture,
    "atlas_texture"
);

// ... use memory ...

tracker.deallocate(id);
```

### RAII-Based Tracking

```rust
use wezterm_metal::TrackedAllocation;

{
    let _alloc = TrackedAllocation::new(
        &tracker,
        4096,
        AllocationType::InstanceBuffer,
        "frame_instances"
    );

    // Memory automatically tracked
} // Automatically deallocated here
```

### Memory Reports

```rust
// Print detailed memory report
tracker.print_report();

// Example output:
// ╔═══════════════════════════════════════════╗
// ║     Memory Report                         ║
// ╠═══════════════════════════════════════════╣
// ║ Total Allocated:  97.3MB                  ║
// ║ Active Allocs:    247                     ║
// ╠═══════════════════════════════════════════╣
// ║ GPU Textures:     8.5MB                   ║
// ║ GPU Buffers:      2.3MB                   ║
// ║ Atlas Entries:    4.2MB                   ║
// ║ Instance Buffers: 1.5MB                   ║
// ║ Heap:             80.8MB                  ║
// ╚═══════════════════════════════════════════╝

// Get programmatic breakdown
let breakdown = tracker.get_breakdown();
println!("GPU memory: {:.1}MB",
         (breakdown.gpu_textures + breakdown.gpu_buffers) as f32 / 1024.0 / 1024.0);
```

### Leak Detection

```rust
// Check for memory leaks
let leaks = tracker.check_leaks();

if !leaks.is_empty() {
    eprintln!("⚠️  {} potential leaks detected", leaks.len());
    for leak in &leaks {
        eprintln!("  - {} bytes ({:?}) at {} (age: {:?})",
                 leak.size,
                 leak.alloc_type,
                 leak.location,
                 leak.age);
    }
}
```

## Performance Alerts

### Configure Alerts

```rust
use wezterm_metal::PerformanceAlerts;

let alerts = PerformanceAlerts {
    frame_time_threshold: 16.7,  // 60 FPS
    drop_rate_threshold: 5.0,     // 5%
    memory_threshold: 200.0,      // 200MB
};

// Check for issues
let issues = alerts.check(&stats, current_frame);
for alert in issues {
    alert.print();
    // Example: ⚠️  [PERF] High frame time: 18.5ms (threshold: 16.7ms)
}
```

## Debugging Workflows

### Finding Performance Bottlenecks

1. **Enable profiling**:
   ```rust
   let debug_config = DebugConfig {
       enable_profiling: true,
       verbose_logging: true,
       ..Default::default()
   };
   ```

2. **Run your workload**:
   ```bash
   cargo run --release
   ```

3. **Check frame breakdown**:
   - Warnings will be printed for slow frames
   - Section timings show where time is spent

4. **Analyze results**:
   ```rust
   let frame = profiler.get_last_frame().unwrap();
   for section in &frame.sections {
       println!("{}: {:.2}ms", section.name,
                section.duration.as_secs_f32() * 1000.0);
   }
   ```

### Debugging Memory Leaks

1. **Enable memory tracking**:
   ```rust
   let tracker = MemoryTracker::new(true);
   ```

2. **Run test scenario**

3. **Check for leaks**:
   ```rust
   let leaks = tracker.check_leaks();
   tracker.print_report();
   ```

4. **Fix identified leaks** using the location information

### GPU Debugging

1. **Enable validation**:
   ```bash
   export METAL_DEVICE_WRAPPER_TYPE=1
   ```

2. **Capture problematic frame**:
   ```rust
   gpu_debugger.start_programmatic_capture(&device)?;
   // Render frame
   gpu_debugger.stop_programmatic_capture();
   ```

3. **Open in Xcode GPU Debugger**

4. **Inspect**:
   - Shader execution
   - Texture contents
   - Buffer data
   - Performance counters

## Advanced Debugging

### Custom Instrumentation Points

```rust
// Add custom markers
profiler.record_section("custom_operation", duration, TimingType::Cpu);

// Record GPU timing
let gpu_start = command_buffer.gpu_start_time();
let gpu_end = command_buffer.gpu_end_time();
profiler.record_gpu_time(gpu_end - gpu_start);
```

### Performance Regression Detection

```rust
// Save baseline
let baseline = profiler.get_stats();

// ... make changes ...

// Compare
let current = profiler.get_stats();
if current.avg_frame_time_ms > baseline.avg_frame_time_ms * 1.1 {
    eprintln!("⚠️  Performance regression: {:.1}% slower",
             ((current.avg_frame_time_ms / baseline.avg_frame_time_ms) - 1.0) * 100.0);
}
```

### Continuous Monitoring

```rust
use std::time::Duration;

// Monitor performance over time
loop {
    let stats = profiler.get_stats();

    if stats.avg_frame_time_ms > 16.7 {
        eprintln!("⚠️  Performance degradation detected");
        stats.print();
    }

    std::thread::sleep(Duration::from_secs(1));
}
```

## Environment Variables

### Debug Configuration

```bash
# Enable Metal validation
export METAL_DEVICE_WRAPPER_TYPE=1

# Enable shader validation
export METAL_SHADER_VALIDATION=1

# Capture to GPU trace
export METAL_CAPTURE_ENABLED=1

# Show shader compiler warnings
export METAL_COMPILER_WARNINGS=1

# Enable API validation
export METAL_API_VALIDATION=1
```

### Performance Tuning

```bash
# Disable validation in production
unset METAL_DEVICE_WRAPPER_TYPE

# Reduce logging
export RUST_LOG=error

# Enable release optimizations
cargo build --release
```

## Troubleshooting

### High Frame Times

1. Check section timings
2. Profile with Instruments.app
3. Look for CPU-GPU synchronization stalls
4. Check for excessive allocations

### Memory Leaks

1. Enable memory tracking
2. Run leak detection
3. Check allocation/deallocation pairs
4. Use RAII patterns (TrackedAllocation)

### GPU Errors

1. Enable validation layer
2. Check shader compilation errors
3. Verify resource binding
4. Check pipeline state

### Validation Errors

Common issues:
- Uninitialized textures
- Invalid pipeline state
- Missing resource binding
- Incorrect buffer sizes

## Best Practices

1. **Always profile in Release mode** for accurate timings
2. **Use RAII guards** for automatic cleanup
3. **Enable validation** during development
4. **Disable debug features** in production
5. **Monitor continuously** for regressions
6. **Document performance targets** and track against them
7. **Use frame capture** for visual debugging
8. **Track memory** to prevent leaks

## Tools Integration

### Instruments.app

```bash
# Profile with Instruments
instruments -t "Metal System Trace" ./target/release/wezterm
```

### Xcode GPU Debugger

1. Product > Profile
2. Select "Metal System Trace"
3. Analyze frame timing, shader performance, memory usage

### Console.app

View Metal validation errors and warnings in real-time.

## Performance Targets

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| FPS | 120 | < 60 |
| Frame time | <8.3ms | > 16.7ms |
| P95 frame time | <12ms | > 20ms |
| P99 frame time | <16ms | > 25ms |
| Drop rate | <1% | > 5% |
| Memory | <100MB | > 200MB |
| Draw calls | 1 | > 5 |

Monitor these metrics continuously to ensure optimal performance.
