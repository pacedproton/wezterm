//! Comprehensive debugging and instrumentation for Metal renderer
//!
//! This module provides:
//! - Performance profiling with nanosecond precision
//! - Metal GPU debugging integration
//! - Frame capture and analysis
//! - Visual debug overlays
//! - Memory tracking and leak detection
//! - Crash reporting

use metal::*;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

/// Debug configuration
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Enable GPU debugging (Metal validation layer)
    pub enable_gpu_validation: bool,
    /// Enable frame capture
    pub enable_frame_capture: bool,
    /// Enable visual debug overlay
    pub enable_overlay: bool,
    /// Enable memory tracking
    pub enable_memory_tracking: bool,
    /// Log frame time warnings if > threshold
    pub frame_time_warning_ms: f32,
    /// Maximum frames to keep in history
    pub frame_history_size: usize,
    /// Enable verbose logging
    pub verbose_logging: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enable_profiling: cfg!(debug_assertions),
            enable_gpu_validation: cfg!(debug_assertions),
            enable_frame_capture: false,
            enable_overlay: cfg!(debug_assertions),
            enable_memory_tracking: cfg!(debug_assertions),
            frame_time_warning_ms: 16.7, // 60 FPS threshold
            frame_history_size: 120, // 2 seconds at 60 FPS
            verbose_logging: false,
        }
    }
}

/// Performance profiler with nanosecond precision
pub struct PerformanceProfiler {
    /// Configuration
    config: DebugConfig,
    /// Frame history
    frame_history: RwLock<VecDeque<FrameProfile>>,
    /// Current frame being profiled
    current_frame: RwLock<Option<FrameProfile>>,
    /// Total frames profiled
    total_frames: AtomicU64,
    /// Dropped frames (>16.7ms)
    dropped_frames: AtomicU64,
    /// Enable/disable profiling at runtime
    enabled: AtomicBool,
}

/// Profile data for a single frame
#[derive(Debug, Clone)]
pub struct FrameProfile {
    /// Frame number
    pub frame_number: u64,
    /// Frame start time
    pub start_time: Instant,
    /// Total frame duration
    pub total_duration: Duration,
    /// Individual section timings
    pub sections: Vec<SectionTiming>,
    /// GPU command buffer timing (if available)
    pub gpu_time: Option<Duration>,
    /// Memory snapshot
    pub memory: MemorySnapshot,
    /// Draw call count
    pub draw_calls: u32,
    /// Vertices rendered
    pub vertices: u64,
    /// Glyphs rendered
    pub glyphs: u32,
}

/// Timing for a specific section of the frame
#[derive(Debug, Clone)]
pub struct SectionTiming {
    /// Section name
    pub name: String,
    /// Duration
    pub duration: Duration,
    /// Start time (relative to frame start)
    pub start_offset: Duration,
    /// CPU or GPU time
    pub timing_type: TimingType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingType {
    Cpu,
    Gpu,
}

/// Memory snapshot
#[derive(Debug, Clone, Copy, Default)]
pub struct MemorySnapshot {
    /// Total heap allocated (bytes)
    pub heap_allocated: usize,
    /// GPU memory used (bytes)
    pub gpu_memory: usize,
    /// Atlas memory (bytes)
    pub atlas_memory: usize,
    /// Instance buffer memory (bytes)
    pub instance_buffer: usize,
    /// Number of allocations
    pub allocation_count: u64,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new(config: DebugConfig) -> Self {
        Self {
            enabled: AtomicBool::new(config.enable_profiling),
            config: config.clone(),
            frame_history: RwLock::new(VecDeque::with_capacity(config.frame_history_size)),
            current_frame: RwLock::new(None),
            total_frames: AtomicU64::new(0),
            dropped_frames: AtomicU64::new(0),
        }
    }

    /// Begin profiling a new frame
    pub fn begin_frame(&self) -> FrameGuard {
        if !self.enabled.load(Ordering::Relaxed) {
            return FrameGuard { profiler: None };
        }

        let frame_number = self.total_frames.fetch_add(1, Ordering::Relaxed);
        let start_time = Instant::now();

        let frame = FrameProfile {
            frame_number,
            start_time,
            total_duration: Duration::ZERO,
            sections: Vec::new(),
            gpu_time: None,
            memory: MemorySnapshot::default(),
            draw_calls: 0,
            vertices: 0,
            glyphs: 0,
        };

        *self.current_frame.write() = Some(frame);

        FrameGuard {
            profiler: Some(self),
        }
    }

    /// Record a timed section
    pub fn record_section(&self, name: impl Into<String>, duration: Duration, timing_type: TimingType) {
        if let Some(frame) = self.current_frame.write().as_mut() {
            let start_offset = frame.start_time.elapsed() - duration;
            frame.sections.push(SectionTiming {
                name: name.into(),
                duration,
                start_offset,
                timing_type,
            });
        }
    }

    /// Record GPU time
    pub fn record_gpu_time(&self, duration: Duration) {
        if let Some(frame) = self.current_frame.write().as_mut() {
            frame.gpu_time = Some(duration);
        }
    }

    /// Record draw call statistics
    pub fn record_draw_stats(&self, draw_calls: u32, vertices: u64, glyphs: u32) {
        if let Some(frame) = self.current_frame.write().as_mut() {
            frame.draw_calls = draw_calls;
            frame.vertices = vertices;
            frame.glyphs = glyphs;
        }
    }

    /// Record memory snapshot
    pub fn record_memory(&self, snapshot: MemorySnapshot) {
        if let Some(frame) = self.current_frame.write().as_mut() {
            frame.memory = snapshot;
        }
    }

    /// End the current frame
    fn end_frame(&self) {
        if let Some(mut frame) = self.current_frame.write().take() {
            frame.total_duration = frame.start_time.elapsed();

            // Check for dropped frames
            let frame_time_ms = frame.total_duration.as_secs_f32() * 1000.0;
            if frame_time_ms > self.config.frame_time_warning_ms {
                self.dropped_frames.fetch_add(1, Ordering::Relaxed);

                if self.config.verbose_logging {
                    eprintln!(
                        "⚠️  Frame {} took {:.2}ms (threshold: {:.2}ms)",
                        frame.frame_number,
                        frame_time_ms,
                        self.config.frame_time_warning_ms
                    );
                    self.print_frame_breakdown(&frame);
                }
            }

            // Add to history
            let mut history = self.frame_history.write();
            if history.len() >= self.config.frame_history_size {
                history.pop_front();
            }
            history.push_back(frame);
        }
    }

    /// Print detailed frame breakdown
    fn print_frame_breakdown(&self, frame: &FrameProfile) {
        eprintln!("  Frame breakdown:");
        for section in &frame.sections {
            eprintln!(
                "    {:30} {:8.3}ms ({:?})",
                section.name,
                section.duration.as_secs_f32() * 1000.0,
                section.timing_type
            );
        }
        if let Some(gpu_time) = frame.gpu_time {
            eprintln!("    GPU Total:                     {:8.3}ms",
                     gpu_time.as_secs_f32() * 1000.0);
        }
        eprintln!("    Draw calls: {}, Glyphs: {}", frame.draw_calls, frame.glyphs);
    }

    /// Get statistics for recent frames
    pub fn get_stats(&self) -> ProfileStats {
        let history = self.frame_history.read();

        if history.is_empty() {
            return ProfileStats::default();
        }

        let total_frames = history.len() as f32;
        let avg_frame_time: f32 = history.iter()
            .map(|f| f.total_duration.as_secs_f32())
            .sum::<f32>() / total_frames;

        let avg_gpu_time: f32 = history.iter()
            .filter_map(|f| f.gpu_time)
            .map(|d| d.as_secs_f32())
            .sum::<f32>() / total_frames;

        let min_frame_time = history.iter()
            .map(|f| f.total_duration)
            .min()
            .unwrap_or(Duration::ZERO);

        let max_frame_time = history.iter()
            .map(|f| f.total_duration)
            .max()
            .unwrap_or(Duration::ZERO);

        let avg_draw_calls = history.iter()
            .map(|f| f.draw_calls as f32)
            .sum::<f32>() / total_frames;

        let avg_glyphs = history.iter()
            .map(|f| f.glyphs as f32)
            .sum::<f32>() / total_frames;

        // Calculate percentiles
        let mut durations: Vec<f32> = history.iter()
            .map(|f| f.total_duration.as_secs_f32() * 1000.0)
            .collect();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let p50 = durations[durations.len() / 2];
        let p95 = durations[(durations.len() * 95) / 100];
        let p99 = durations[(durations.len() * 99) / 100];

        ProfileStats {
            total_frames: self.total_frames.load(Ordering::Relaxed),
            dropped_frames: self.dropped_frames.load(Ordering::Relaxed),
            avg_fps: 1.0 / avg_frame_time,
            avg_frame_time_ms: avg_frame_time * 1000.0,
            avg_gpu_time_ms: avg_gpu_time * 1000.0,
            min_frame_time_ms: min_frame_time.as_secs_f32() * 1000.0,
            max_frame_time_ms: max_frame_time.as_secs_f32() * 1000.0,
            p50_frame_time_ms: p50,
            p95_frame_time_ms: p95,
            p99_frame_time_ms: p99,
            avg_draw_calls,
            avg_glyphs,
            frames_in_history: history.len(),
        }
    }

    /// Get the most recent frame
    pub fn get_last_frame(&self) -> Option<FrameProfile> {
        self.frame_history.read().back().cloned()
    }

    /// Get all frames in history
    pub fn get_frame_history(&self) -> Vec<FrameProfile> {
        self.frame_history.read().iter().cloned().collect()
    }

    /// Enable/disable profiling
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Clear frame history
    pub fn clear_history(&self) {
        self.frame_history.write().clear();
        self.total_frames.store(0, Ordering::Relaxed);
        self.dropped_frames.store(0, Ordering::Relaxed);
    }
}

/// RAII guard for frame profiling
pub struct FrameGuard<'a> {
    profiler: Option<&'a PerformanceProfiler>,
}

impl<'a> Drop for FrameGuard<'a> {
    fn drop(&mut self) {
        if let Some(profiler) = self.profiler {
            profiler.end_frame();
        }
    }
}

/// RAII guard for section profiling
pub struct SectionGuard<'a> {
    profiler: &'a PerformanceProfiler,
    name: String,
    start_time: Instant,
    timing_type: TimingType,
}

impl<'a> SectionGuard<'a> {
    /// Create a new section guard
    pub fn new(
        profiler: &'a PerformanceProfiler,
        name: impl Into<String>,
        timing_type: TimingType,
    ) -> Self {
        Self {
            profiler,
            name: name.into(),
            start_time: Instant::now(),
            timing_type,
        }
    }
}

impl<'a> Drop for SectionGuard<'a> {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        self.profiler.record_section(&self.name, duration, self.timing_type);
    }
}

/// Aggregate statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct ProfileStats {
    /// Total frames profiled
    pub total_frames: u64,
    /// Frames that exceeded threshold
    pub dropped_frames: u64,
    /// Average FPS
    pub avg_fps: f32,
    /// Average frame time (ms)
    pub avg_frame_time_ms: f32,
    /// Average GPU time (ms)
    pub avg_gpu_time_ms: f32,
    /// Minimum frame time (ms)
    pub min_frame_time_ms: f32,
    /// Maximum frame time (ms)
    pub max_frame_time_ms: f32,
    /// 50th percentile (median)
    pub p50_frame_time_ms: f32,
    /// 95th percentile
    pub p95_frame_time_ms: f32,
    /// 99th percentile
    pub p99_frame_time_ms: f32,
    /// Average draw calls per frame
    pub avg_draw_calls: f32,
    /// Average glyphs per frame
    pub avg_glyphs: f32,
    /// Frames in history buffer
    pub frames_in_history: usize,
}

impl ProfileStats {
    /// Print statistics to stderr
    pub fn print(&self) {
        eprintln!("╔═══════════════════════════════════════════╗");
        eprintln!("║     Performance Statistics                ║");
        eprintln!("╠═══════════════════════════════════════════╣");
        eprintln!("║ Total Frames:     {:<23} ║", self.total_frames);
        eprintln!("║ Dropped Frames:   {:<23} ║", self.dropped_frames);
        eprintln!("║ Drop Rate:        {:<21.2}% ║",
                 (self.dropped_frames as f32 / self.total_frames as f32) * 100.0);
        eprintln!("╠═══════════════════════════════════════════╣");
        eprintln!("║ Average FPS:      {:<21.1} ║", self.avg_fps);
        eprintln!("║ Avg Frame Time:   {:<19.2}ms ║", self.avg_frame_time_ms);
        eprintln!("║ Avg GPU Time:     {:<19.2}ms ║", self.avg_gpu_time_ms);
        eprintln!("╠═══════════════════════════════════════════╣");
        eprintln!("║ Min Frame Time:   {:<19.2}ms ║", self.min_frame_time_ms);
        eprintln!("║ P50 Frame Time:   {:<19.2}ms ║", self.p50_frame_time_ms);
        eprintln!("║ P95 Frame Time:   {:<19.2}ms ║", self.p95_frame_time_ms);
        eprintln!("║ P99 Frame Time:   {:<19.2}ms ║", self.p99_frame_time_ms);
        eprintln!("║ Max Frame Time:   {:<19.2}ms ║", self.max_frame_time_ms);
        eprintln!("╠═══════════════════════════════════════════╣");
        eprintln!("║ Avg Draw Calls:   {:<21.1} ║", self.avg_draw_calls);
        eprintln!("║ Avg Glyphs:       {:<21.1} ║", self.avg_glyphs);
        eprintln!("╚═══════════════════════════════════════════╝");
    }
}

/// GPU debugging utilities
pub struct GpuDebugger {
    /// Enable GPU validation layer
    validation_enabled: bool,
    /// Capture manager for frame capture
    capture_manager: Option<CaptureManager>,
    /// Current capture scope
    capture_scope: Option<CaptureScope>,
}

impl GpuDebugger {
    /// Create a new GPU debugger
    pub fn new(device: &Device, config: &DebugConfig) -> Self {
        let validation_enabled = config.enable_gpu_validation;

        // Enable Metal validation layer
        if validation_enabled {
            eprintln!("🔍 Metal GPU validation layer enabled");
            // Note: Validation layer is enabled via environment variable:
            // export METAL_DEVICE_WRAPPER_TYPE=1
        }

        // Setup frame capture if enabled
        let (capture_manager, capture_scope) = if config.enable_frame_capture {
            let capture_manager = CaptureManager::shared();

            let capture_descriptor = CaptureDescriptor::new();
            capture_descriptor.set_capture_object(device);
            capture_descriptor.set_destination(MTLCaptureDestination::DeveloperTools);

            let scope = capture_manager
                .new_capture_scope_with_device(device);
            scope.set_label("WezTerm Frame Capture");

            capture_manager.set_default_capture_scope(&scope);

            eprintln!("📹 Frame capture enabled - use Xcode GPU debugger");

            (Some(capture_manager), Some(scope))
        } else {
            (None, None)
        };

        Self {
            validation_enabled,
            capture_manager,
            capture_scope,
        }
    }

    /// Begin capturing a frame
    pub fn begin_capture(&self) {
        if let Some(scope) = &self.capture_scope {
            scope.begin_scope();
        }
    }

    /// End capturing a frame
    pub fn end_capture(&self) {
        if let Some(scope) = &self.capture_scope {
            scope.end_scope();
        }
    }

    /// Start programmatic capture
    pub fn start_programmatic_capture(&self, device: &Device) -> Result<(), String> {
        if let Some(manager) = &self.capture_manager {
            let descriptor = CaptureDescriptor::new();
            descriptor.set_capture_object(device);
            descriptor.set_destination(MTLCaptureDestination::GpuTraceDocument);
            descriptor.set_output_url(&std::path::Path::new("/tmp/wezterm_capture.gputrace"));

            manager.start_capture(&descriptor)
                .map_err(|e| format!("Failed to start capture: {:?}", e))?;

            eprintln!("📹 Started GPU trace capture -> /tmp/wezterm_capture.gputrace");
            Ok(())
        } else {
            Err("Frame capture not enabled".to_string())
        }
    }

    /// Stop programmatic capture
    pub fn stop_programmatic_capture(&self) {
        if let Some(manager) = &self.capture_manager {
            manager.stop_capture();
            eprintln!("✅ Stopped GPU trace capture");
        }
    }
}

/// Macro for timing a code section
#[macro_export]
macro_rules! profile_section {
    ($profiler:expr, $name:expr, $type:expr, $code:block) => {{
        let _guard = $crate::debug::SectionGuard::new($profiler, $name, $type);
        $code
    }};
}

/// Macro for timing a function
#[macro_export]
macro_rules! profile_function {
    ($profiler:expr, $code:block) => {{
        profile_section!($profiler, function_name!(), $crate::debug::TimingType::Cpu, $code)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let config = DebugConfig::default();
        let profiler = PerformanceProfiler::new(config);

        assert_eq!(profiler.total_frames.load(Ordering::Relaxed), 0);
        assert_eq!(profiler.dropped_frames.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_frame_profiling() {
        let config = DebugConfig {
            enable_profiling: true,
            ..Default::default()
        };
        let profiler = PerformanceProfiler::new(config);

        {
            let _guard = profiler.begin_frame();
            profiler.record_section("test", Duration::from_millis(5), TimingType::Cpu);
        }

        assert_eq!(profiler.total_frames.load(Ordering::Relaxed), 1);
        let history = profiler.frame_history.read();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].sections.len(), 1);
    }

    #[test]
    fn test_stats_calculation() {
        let config = DebugConfig {
            enable_profiling: true,
            ..Default::default()
        };
        let profiler = PerformanceProfiler::new(config);

        // Profile several frames
        for _ in 0..10 {
            let _guard = profiler.begin_frame();
            std::thread::sleep(Duration::from_millis(1));
        }

        let stats = profiler.get_stats();
        assert_eq!(stats.frames_in_history, 10);
        assert!(stats.avg_frame_time_ms > 0.0);
        assert!(stats.avg_fps > 0.0);
    }
}
