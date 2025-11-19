//! Visual debug overlay for on-screen performance monitoring
//!
//! Displays real-time performance metrics, frame graphs, and diagnostic info

use crate::debug::{ProfileStats, FrameProfile};
use std::collections::VecDeque;

/// Visual debug overlay
pub struct DebugOverlay {
    /// Show FPS counter
    pub show_fps: bool,
    /// Show frame time graph
    pub show_frame_graph: bool,
    /// Show memory stats
    pub show_memory: bool,
    /// Show GPU stats
    pub show_gpu: bool,
    /// Show detailed breakdown
    pub show_breakdown: bool,
    /// Position on screen
    pub position: OverlayPosition,
    /// Opacity (0.0 - 1.0)
    pub opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self {
            show_fps: true,
            show_frame_graph: true,
            show_memory: false,
            show_gpu: false,
            show_breakdown: false,
            position: OverlayPosition::TopRight,
            opacity: 0.8,
        }
    }
}

impl DebugOverlay {
    /// Generate overlay text
    pub fn generate_text(&self, stats: &ProfileStats, current_frame: Option<&FrameProfile>) -> String {
        let mut text = String::new();

        // FPS counter
        if self.show_fps {
            text.push_str(&format!("FPS: {:.1}\n", stats.avg_fps));
            text.push_str(&format!("Frame: {:.2}ms\n", stats.avg_frame_time_ms));

            // Color code based on performance
            if stats.avg_frame_time_ms > 16.7 {
                text.push_str("⚠️  SLOW\n");
            } else if stats.avg_frame_time_ms > 8.3 {
                text.push_str("⚡ OK\n");
            } else {
                text.push_str("✨ FAST\n");
            }
        }

        // Frame time graph
        if self.show_frame_graph {
            text.push_str("\n");
            text.push_str(&self.generate_sparkline(stats));
            text.push_str("\n");
        }

        // Memory stats
        if self.show_memory {
            if let Some(frame) = current_frame {
                text.push_str("\nMemory:\n");
                text.push_str(&format!("  Heap: {:.1}MB\n",
                    frame.memory.heap_allocated as f32 / 1024.0 / 1024.0));
                text.push_str(&format!("  GPU: {:.1}MB\n",
                    frame.memory.gpu_memory as f32 / 1024.0 / 1024.0));
                text.push_str(&format!("  Atlas: {:.1}MB\n",
                    frame.memory.atlas_memory as f32 / 1024.0 / 1024.0));
            }
        }

        // GPU stats
        if self.show_gpu {
            text.push_str("\nGPU:\n");
            text.push_str(&format!("  Time: {:.2}ms\n", stats.avg_gpu_time_ms));
            text.push_str(&format!("  Draws: {:.0}\n", stats.avg_draw_calls));
            text.push_str(&format!("  Glyphs: {:.0}\n", stats.avg_glyphs));
        }

        // Detailed breakdown
        if self.show_breakdown {
            if let Some(frame) = current_frame {
                text.push_str("\nBreakdown:\n");
                for section in &frame.sections {
                    text.push_str(&format!("  {}: {:.2}ms\n",
                        section.name,
                        section.duration.as_secs_f32() * 1000.0));
                }
            }
        }

        text
    }

    /// Generate ASCII sparkline for frame times
    fn generate_sparkline(&self, stats: &ProfileStats) -> String {
        // Simplified - would need frame history
        let bars = "▁▂▃▄▅▆▇█";
        let mut line = String::from("Frame times: ");

        // Just show a simple pattern for now
        for i in 0..20 {
            let idx = (i % bars.len()) as usize;
            line.push(bars.chars().nth(idx).unwrap());
        }

        line
    }

    /// Render overlay (would integrate with Metal renderer)
    pub fn render_to_string(&self, stats: &ProfileStats, current_frame: Option<&FrameProfile>) -> String {
        self.generate_text(stats, current_frame)
    }
}

/// Frame graph for visualizing performance over time
pub struct FrameGraph {
    /// Frame time history
    frame_times: VecDeque<f32>,
    /// Maximum samples
    max_samples: usize,
    /// Graph height in pixels
    height: u32,
    /// Graph width in pixels
    width: u32,
}

impl FrameGraph {
    /// Create a new frame graph
    pub fn new(max_samples: usize, width: u32, height: u32) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(max_samples),
            max_samples,
            height,
            width,
        }
    }

    /// Add a frame time sample
    pub fn add_sample(&mut self, frame_time_ms: f32) {
        if self.frame_times.len() >= self.max_samples {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(frame_time_ms);
    }

    /// Generate ASCII graph
    pub fn to_ascii(&self) -> String {
        let mut output = String::new();

        if self.frame_times.is_empty() {
            return output;
        }

        let max_time = self.frame_times.iter().cloned().fold(0.0f32, f32::max);
        let height = 10; // ASCII lines

        for row in (0..height).rev() {
            let threshold = (row as f32 / height as f32) * max_time;

            for &time in &self.frame_times {
                if time >= threshold {
                    output.push('█');
                } else {
                    output.push(' ');
                }
            }
            output.push('\n');
        }

        output
    }
}

/// Performance alerts
pub struct PerformanceAlerts {
    /// Alert threshold for frame time (ms)
    pub frame_time_threshold: f32,
    /// Alert threshold for dropped frames (percentage)
    pub drop_rate_threshold: f32,
    /// Alert threshold for memory (MB)
    pub memory_threshold: f32,
}

impl Default for PerformanceAlerts {
    fn default() -> Self {
        Self {
            frame_time_threshold: 16.7, // 60 FPS
            drop_rate_threshold: 5.0, // 5% drop rate
            memory_threshold: 200.0, // 200MB
        }
    }
}

impl PerformanceAlerts {
    /// Check for performance issues
    pub fn check(&self, stats: &ProfileStats, current_frame: Option<&FrameProfile>) -> Vec<Alert> {
        let mut alerts = Vec::new();

        // Check frame time
        if stats.avg_frame_time_ms > self.frame_time_threshold {
            alerts.push(Alert {
                level: AlertLevel::Warning,
                message: format!(
                    "High frame time: {:.2}ms (threshold: {:.2}ms)",
                    stats.avg_frame_time_ms,
                    self.frame_time_threshold
                ),
                category: AlertCategory::Performance,
            });
        }

        // Check drop rate
        let drop_rate = (stats.dropped_frames as f32 / stats.total_frames as f32) * 100.0;
        if drop_rate > self.drop_rate_threshold {
            alerts.push(Alert {
                level: AlertLevel::Error,
                message: format!(
                    "High drop rate: {:.1}% (threshold: {:.1}%)",
                    drop_rate,
                    self.drop_rate_threshold
                ),
                category: AlertCategory::Performance,
            });
        }

        // Check memory
        if let Some(frame) = current_frame {
            let total_memory_mb = (frame.memory.heap_allocated + frame.memory.gpu_memory) as f32 / 1024.0 / 1024.0;
            if total_memory_mb > self.memory_threshold {
                alerts.push(Alert {
                    level: AlertLevel::Warning,
                    message: format!(
                        "High memory usage: {:.1}MB (threshold: {:.1}MB)",
                        total_memory_mb,
                        self.memory_threshold
                    ),
                    category: AlertCategory::Memory,
                });
            }
        }

        alerts
    }
}

/// Performance alert
#[derive(Debug, Clone)]
pub struct Alert {
    /// Alert severity
    pub level: AlertLevel,
    /// Alert message
    pub message: String,
    /// Alert category
    pub category: AlertCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertCategory {
    Performance,
    Memory,
    Gpu,
    Resource,
}

impl Alert {
    /// Get emoji for alert level
    pub fn emoji(&self) -> &'static str {
        match self.level {
            AlertLevel::Info => "ℹ️ ",
            AlertLevel::Warning => "⚠️ ",
            AlertLevel::Error => "❌",
            AlertLevel::Critical => "🔥",
        }
    }

    /// Print alert to stderr
    pub fn print(&self) {
        eprintln!("{} [{}] {}", self.emoji(), self.category_str(), self.message);
    }

    fn category_str(&self) -> &'static str {
        match self.category {
            AlertCategory::Performance => "PERF",
            AlertCategory::Memory => "MEM",
            AlertCategory::Gpu => "GPU",
            AlertCategory::Resource => "RSRC",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_generation() {
        let overlay = DebugOverlay::default();
        let stats = ProfileStats {
            avg_fps: 120.0,
            avg_frame_time_ms: 8.3,
            ..Default::default()
        };

        let text = overlay.generate_text(&stats, None);
        assert!(text.contains("FPS: 120"));
        assert!(text.contains("8.3"));
    }

    #[test]
    fn test_frame_graph() {
        let mut graph = FrameGraph::new(100, 800, 200);

        for i in 0..50 {
            graph.add_sample(8.0 + (i % 10) as f32);
        }

        assert_eq!(graph.frame_times.len(), 50);
    }

    #[test]
    fn test_performance_alerts() {
        let alerts = PerformanceAlerts::default();
        let stats = ProfileStats {
            avg_frame_time_ms: 20.0, // Above threshold
            total_frames: 100,
            dropped_frames: 10, // 10% drop rate
            ..Default::default()
        };

        let alerts_vec = alerts.check(&stats, None);
        assert_eq!(alerts_vec.len(), 2); // Frame time + drop rate
    }
}
