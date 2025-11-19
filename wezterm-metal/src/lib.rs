//! High-performance Metal renderer for WezTerm
//!
//! This renderer is designed to achieve:
//! - 120 FPS on ProMotion displays
//! - <5ms input latency
//! - Zero-copy rendering from terminal buffer
//! - GPU-accelerated glyph rendering
//! - Minimal CPU usage

#![cfg(target_os = "macos")]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod atlas;
pub mod blit;
pub mod debug;
pub mod debug_overlay;
pub mod memory_tracker;
pub mod pipeline;
pub mod renderer;
pub mod shaders;

#[cfg(target_arch = "aarch64")]
pub mod simd;

pub use atlas::GlyphAtlas;
pub use blit::{Blitter, DirtyRect, DirtyTracker, TripleBuffer};
pub use debug::{DebugConfig, PerformanceProfiler, ProfileStats, GpuDebugger};
pub use debug_overlay::{DebugOverlay, FrameGraph, PerformanceAlerts};
pub use memory_tracker::{MemoryTracker, MemoryBreakdown, AllocationType};
pub use pipeline::RenderPipeline;
pub use renderer::{MetalRenderer, RenderConfig, RenderStats};

/// Performance metrics
#[derive(Debug, Clone, Copy, Default)]
pub struct PerformanceMetrics {
    /// Frames per second
    pub fps: f32,
    /// Frame time in milliseconds
    pub frame_time_ms: f32,
    /// GPU time in milliseconds
    pub gpu_time_ms: f32,
    /// CPU time in milliseconds
    pub cpu_time_ms: f32,
    /// Number of draw calls per frame
    pub draw_calls: u32,
    /// Number of glyphs rendered
    pub glyphs_rendered: u32,
    /// Memory used by glyph atlas in MB
    pub atlas_memory_mb: f32,
}

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
