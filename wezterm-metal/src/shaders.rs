//! Metal shader utilities and constants
//!
//! This module contains shader management and utilities for the Metal renderer

/// Shader source code (Metal Shading Language)
pub const GLYPH_SHADER_MSL: &str = include_str!("shaders/glyph.metal");

/// Shader compiler options
pub struct ShaderCompileOptions {
    /// Enable fast math optimizations
    pub fast_math: bool,
    /// Optimization level (0-3)
    pub optimization_level: u32,
}

impl Default for ShaderCompileOptions {
    fn default() -> Self {
        Self {
            fast_math: true,
            optimization_level: 3,
        }
    }
}
