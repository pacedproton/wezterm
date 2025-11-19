//! Debug and instrumentation demo
//!
//! Demonstrates all debugging features of the Metal renderer:
//! - Performance profiling
//! - Memory tracking
//! - Visual overlays
//! - GPU debugging
//! - Alert system
//!
//! Run with:
//! ```bash
//! # Enable GPU validation
//! export METAL_DEVICE_WRAPPER_TYPE=1
//!
//! # Run the demo
//! cargo run --release --example debug_demo
//! ```

use std::time::Duration;

fn main() {
    println!("╔══════════════════════════════════════════════╗");
    println!("║   WezTerm Metal Renderer Debug Demo         ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();

    // Demonstrate debug configuration
    demo_debug_config();

    // Demonstrate performance profiling
    demo_performance_profiling();

    // Demonstrate memory tracking
    demo_memory_tracking();

    // Demonstrate visual overlay
    demo_visual_overlay();

    // Demonstrate alerts
    demo_alerts();

    // Demonstrate frame graph
    demo_frame_graph();

    println!("\n✅ Debug demo complete!");
}

fn demo_debug_config() {
    println!("📋 Debug Configuration");
    println!("─────────────────────");

    #[cfg(target_os = "macos")]
    {
        use wezterm_metal::DebugConfig;

        let config = DebugConfig {
            enable_profiling: true,
            enable_gpu_validation: true,
            enable_frame_capture: true,
            enable_overlay: true,
            enable_memory_tracking: true,
            frame_time_warning_ms: 16.7,
            frame_history_size: 120,
            verbose_logging: true,
        };

        println!("  ✓ Profiling: {}", config.enable_profiling);
        println!("  ✓ GPU Validation: {}", config.enable_gpu_validation);
        println!("  ✓ Frame Capture: {}", config.enable_frame_capture);
        println!("  ✓ Overlay: {}", config.enable_overlay);
        println!("  ✓ Memory Tracking: {}", config.enable_memory_tracking);
        println!("  ✓ Frame Warning Threshold: {:.1}ms", config.frame_time_warning_ms);
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("  ⚠️  Debug features only available on macOS");
    }

    println!();
}

fn demo_performance_profiling() {
    println!("📊 Performance Profiling");
    println!("────────────────────────");

    #[cfg(target_os = "macos")]
    {
        use wezterm_metal::{DebugConfig, PerformanceProfiler};
        use wezterm_metal::debug::TimingType;

        let config = DebugConfig::default();
        let profiler = PerformanceProfiler::new(config);

        // Simulate rendering 100 frames
        for frame_num in 0..100 {
            let _guard = profiler.begin_frame();

            // Simulate some work
            {
                let _section = profiler.begin_frame();
                std::thread::sleep(Duration::from_micros(100));
            }

            profiler.record_section("cell_processing", Duration::from_micros(500), TimingType::Cpu);
            profiler.record_section("glyph_upload", Duration::from_micros(200), TimingType::Gpu);
            profiler.record_section("draw_call", Duration::from_micros(100), TimingType::Gpu);

            profiler.record_draw_stats(1, 4096, 1024);

            if frame_num % 10 == 0 {
                std::thread::sleep(Duration::from_millis(5)); // Simulate occasional slowdown
            }
        }

        // Print statistics
        println!();
        let stats = profiler.get_stats();
        stats.print();
        println!();
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("  ⚠️  Profiling only available on macOS");
    }

    println!();
}

fn demo_memory_tracking() {
    println!("💾 Memory Tracking");
    println!("──────────────────");

    #[cfg(target_os = "macos")]
    {
        use wezterm_metal::{MemoryTracker, AllocationType};

        let tracker = MemoryTracker::new(true);

        // Simulate allocations
        let _tex1 = tracker.allocate(8 * 1024 * 1024, AllocationType::GpuTexture, "atlas_texture");
        let _buf1 = tracker.allocate(2 * 1024 * 1024, AllocationType::GpuBuffer, "instance_buffer");
        let _atlas1 = tracker.allocate(4 * 1024 * 1024, AllocationType::AtlasEntry, "glyph_cache");
        let _heap1 = tracker.allocate(512 * 1024, AllocationType::HeapAllocation, "temp_buffer");

        // Print report
        println!();
        tracker.print_report();
        println!();

        // Clean up
        tracker.deallocate(_tex1);
        tracker.deallocate(_buf1);
        tracker.deallocate(_atlas1);
        tracker.deallocate(_heap1);

        println!("  ✓ All memory deallocated");
        println!("  ✓ Current usage: {} bytes", tracker.current_usage());
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("  ⚠️  Memory tracking only available on macOS");
    }

    println!();
}

fn demo_visual_overlay() {
    println!("👁️  Visual Debug Overlay");
    println!("────────────────────────");

    #[cfg(target_os = "macos")]
    {
        use wezterm_metal::{DebugOverlay, ProfileStats};

        let overlay = DebugOverlay::default();

        let stats = ProfileStats {
            total_frames: 1000,
            dropped_frames: 12,
            avg_fps: 118.5,
            avg_frame_time_ms: 8.45,
            avg_gpu_time_ms: 1.23,
            min_frame_time_ms: 7.2,
            max_frame_time_ms: 15.8,
            p50_frame_time_ms: 8.3,
            p95_frame_time_ms: 12.1,
            p99_frame_time_ms: 14.5,
            avg_draw_calls: 1.0,
            avg_glyphs: 2048.0,
            frames_in_history: 120,
        };

        let text = overlay.render_to_string(&stats, None);
        println!();
        println!("{}", text);
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("  ⚠️  Overlay only available on macOS");
    }

    println!();
}

fn demo_alerts() {
    println!("🚨 Performance Alerts");
    println!("─────────────────────");

    #[cfg(target_os = "macos")]
    {
        use wezterm_metal::{PerformanceAlerts, ProfileStats};

        let alerts = PerformanceAlerts::default();

        // Simulate poor performance
        let bad_stats = ProfileStats {
            total_frames: 1000,
            dropped_frames: 80, // 8% drop rate - bad!
            avg_fps: 55.0, // Below 60 FPS
            avg_frame_time_ms: 18.2, // Above 16.7ms threshold
            avg_gpu_time_ms: 2.5,
            min_frame_time_ms: 15.0,
            max_frame_time_ms: 35.0,
            p50_frame_time_ms: 17.5,
            p95_frame_time_ms: 25.0,
            p99_frame_time_ms: 32.0,
            avg_draw_calls: 1.0,
            avg_glyphs: 2048.0,
            frames_in_history: 120,
        };

        let issues = alerts.check(&bad_stats, None);

        println!("  Detected {} issues:", issues.len());
        println!();
        for alert in issues {
            alert.print();
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("  ⚠️  Alerts only available on macOS");
    }

    println!();
}

fn demo_frame_graph() {
    println!("📈 Frame Time Graph");
    println!("───────────────────");

    #[cfg(target_os = "macos")]
    {
        use wezterm_metal::FrameGraph;
        use rand::Rng;

        let mut graph = FrameGraph::new(60, 800, 100);
        let mut rng = rand::thread_rng();

        // Generate realistic frame times
        for _ in 0..60 {
            let base = 8.0;
            let noise = rng.gen_range(-1.0..2.0);
            let spike = if rng.gen_bool(0.1) { rng.gen_range(5.0..10.0) } else { 0.0 };
            let frame_time = base + noise + spike;

            graph.add_sample(frame_time);
        }

        println!();
        println!("{}", graph.to_ascii());
        println!("  (Each row = ~2ms, 60 frames shown)");
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("  ⚠️  Frame graph only available on macOS");
    }

    println!();
}
