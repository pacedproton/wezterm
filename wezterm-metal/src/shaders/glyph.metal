// High-performance glyph rendering shader for WezTerm
// Optimized for M1/M2/M3 GPUs
//
// Performance characteristics:
// - Single draw call for all glyphs (GPU instancing)
// - Minimal state changes
// - Efficient texture sampling
// - Optimized for tile-based deferred rendering (TBDR)

#include <metal_stdlib>
using namespace metal;

// This file serves as a placeholder for the actual shader source
// The real shaders are compiled from the pipeline.rs SHADER_SOURCE constant
