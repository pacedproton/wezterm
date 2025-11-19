//! Metal render pipeline
//!
//! Optimized for single-pass instanced rendering of all terminal glyphs

use metal::*;

/// Render pipeline for terminal rendering
pub struct RenderPipeline {
    /// Pipeline state object
    pipeline_state: RenderPipelineState,
}

impl RenderPipeline {
    /// Create a new render pipeline
    pub fn new(device: &Device) -> Result<Self, String> {
        // Compile shaders
        let library = device
            .new_library_with_source(SHADER_SOURCE, &CompileOptions::new())
            .map_err(|e| format!("Failed to compile shaders: {}", e))?;

        let vertex_function = library
            .get_function("vertex_main", None)
            .map_err(|e| format!("Vertex function not found: {}", e))?;

        let fragment_function = library
            .get_function("fragment_main", None)
            .map_err(|e| format!("Fragment function not found: {}", e))?;

        // Create pipeline descriptor
        let pipeline_desc = RenderPipelineDescriptor::new();
        pipeline_desc.set_vertex_function(Some(&vertex_function));
        pipeline_desc.set_fragment_function(Some(&fragment_function));

        // Color attachment
        let color_attachment = pipeline_desc
            .color_attachments()
            .object_at(0)
            .unwrap();
        color_attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);

        // Enable blending for transparent glyphs
        color_attachment.set_blending_enabled(true);
        color_attachment.set_rgb_blend_operation(MTLBlendOperation::Add);
        color_attachment.set_alpha_blend_operation(MTLBlendOperation::Add);
        color_attachment.set_source_rgb_blend_factor(MTLBlendFactor::SourceAlpha);
        color_attachment.set_source_alpha_blend_factor(MTLBlendFactor::SourceAlpha);
        color_attachment.set_destination_rgb_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);
        color_attachment.set_destination_alpha_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);

        // Create pipeline state
        let pipeline_state = device
            .new_render_pipeline_state(&pipeline_desc)
            .map_err(|e| format!("Failed to create pipeline state: {}", e))?;

        Ok(Self { pipeline_state })
    }

    /// Get the pipeline state
    pub fn pipeline_state(&self) -> &RenderPipelineState {
        &self.pipeline_state
    }
}

/// Metal Shading Language (MSL) shader source
const SHADER_SOURCE: &str = r#"
#include <metal_stdlib>
using namespace metal;

// Vertex input (per-instance data)
struct InstanceData {
    float2 offset;           // Screen position offset
    float4 atlas_rect;       // Atlas UV coordinates
    float4 fg_color;         // Foreground color (RGBA)
    float4 bg_color;         // Background color (RGBA)
    float4 attrs;            // Attributes (bold, italic, underline, strikethrough)
};

// Vertex output / Fragment input
struct VertexOut {
    float4 position [[position]];
    float2 tex_coord;
    float4 fg_color;
    float4 bg_color;
    float4 attrs;
};

// Vertex shader - generates quad for each glyph instance
vertex VertexOut vertex_main(
    uint vertex_id [[vertex_id]],
    uint instance_id [[instance_id]],
    constant InstanceData* instances [[buffer(0)]]
) {
    InstanceData instance = instances[instance_id];

    // Generate quad vertices (0,0) -> (1,1)
    float2 quad_vertices[4] = {
        float2(0.0, 0.0),  // Top-left
        float2(1.0, 0.0),  // Top-right
        float2(0.0, 1.0),  // Bottom-left
        float2(1.0, 1.0),  // Bottom-right
    };

    float2 quad_pos = quad_vertices[vertex_id];

    // Calculate screen position
    // TODO: Apply projection matrix for proper NDC conversion
    float2 screen_pos = instance.offset + quad_pos * float2(10.0, 20.0); // Cell size

    // Convert to NDC (-1 to 1)
    // Assume screen size 800x600 for now - should be uniform
    float2 ndc = (screen_pos / float2(800.0, 600.0)) * 2.0 - 1.0;
    ndc.y = -ndc.y; // Flip Y

    // Calculate texture coordinates
    float2 tex_coord = instance.atlas_rect.xy + quad_pos * instance.atlas_rect.zw;

    VertexOut out;
    out.position = float4(ndc, 0.0, 1.0);
    out.tex_coord = tex_coord;
    out.fg_color = instance.fg_color;
    out.bg_color = instance.bg_color;
    out.attrs = instance.attrs;

    return out;
}

// Fragment shader - samples glyph from atlas and applies color
fragment float4 fragment_main(
    VertexOut in [[stage_in]],
    texture2d<float> atlas [[texture(0)]],
    sampler atlas_sampler [[sampler(0)]]
) {
    // Sample glyph alpha from atlas
    float4 glyph = atlas.sample(atlas_sampler, in.tex_coord);
    float alpha = glyph.a;

    // Mix background and foreground based on glyph alpha
    float4 color = mix(in.bg_color, in.fg_color, alpha);

    // Apply text attributes
    // Bold: slightly brighten
    if (in.attrs.x > 0.5) {
        color.rgb = min(color.rgb * 1.1, float3(1.0));
    }

    // Italic: handled in vertex shader (skew)

    // Underline/strikethrough: would need separate pass or SDF

    return color;
}
"#;
