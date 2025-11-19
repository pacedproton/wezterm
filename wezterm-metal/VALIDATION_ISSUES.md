# Critical Issues Found in Metal Bit Blitting Implementation

## 🔴 BLOCKER ISSUES (Will Not Compile)

### 1. TripleBuffer::new() Signature Mismatch

**Location**: `wezterm-metal/src/blit.rs:295` and `wezterm-metal/src/renderer.rs:160`

**Issue**: Function signature doesn't match usage

**Current Signature** (blit.rs:295):
```rust
pub fn new(device: &Device, width: u64, height: u64, pixel_format: MTLPixelFormat) -> Self
```

**Current Call** (renderer.rs:160):
```rust
let t_buffer = TripleBuffer::new(&device, config.cols, config.rows, config.cell_width, config.cell_height)?;
```

**Problems**:
- Takes 5 arguments, function expects 4
- `cols` (u16) passed where `width` (u64) expected
- `rows` (u16) passed where `height` (u64) expected
- `cell_width` (f32) passed where `pixel_format` (MTLPixelFormat) expected
- `cell_height` (f32) is extra
- Missing pixel format entirely
- Missing texture dimensions calculation

**Fix Required**:
```rust
// In blit.rs, change signature to:
pub fn new(
    device: &Device,
    cols: u16,
    rows: u16,
    cell_width: f32,
    cell_height: f32,
    pixel_format: MTLPixelFormat,
) -> Result<Self, String>

// Calculate pixel dimensions:
let width = (cols as f32 * cell_width).ceil() as u64;
let height = (rows as f32 * cell_height).ceil() as u64;
```

### 2. Missing Result Return Type

**Location**: `wezterm-metal/src/blit.rs:295`

**Issue**: TripleBuffer::new() returns `Self` but can fail

**Current**:
```rust
pub fn new(...) -> Self
```

**Should be**:
```rust
pub fn new(...) -> Result<Self, String>
```

**Reason**: Metal texture creation can fail (out of memory, invalid dimensions, etc.)

### 3. Missing Pixel Format in Renderer Call

**Location**: `wezterm-metal/src/renderer.rs:160`

**Issue**: No pixel format specified for triple buffer

**Fix**: Need to determine render target pixel format
```rust
// Should be:
let pixel_format = drawable.texture().pixel_format(); // or MTLPixelFormat::BGRA8Unorm
let t_buffer = TripleBuffer::new(
    &device,
    config.cols,
    config.rows,
    config.cell_width,
    config.cell_height,
    pixel_format,
)?;
```

## ⚠️ DESIGN ISSUES (Will Compile But Won't Work)

### 4. Blitter::blit_unchanged() Not Implemented

**Location**: `wezterm-metal/src/blit.rs:394-425`

**Issue**: Function claims to blit "unchanged regions" but actually blits everything

**Current Code**:
```rust
pub fn blit_unchanged(...) {
    if dirty_rects.is_empty() {
        self.blit_full(encoder, src, dst);
        return;
    }

    // ... calculates dirty percentage ...

    if dirty_percentage > 50.0 {
        self.blit_full(encoder, src, dst);
    } else {
        // Blit row by row, skipping dirty rows
        // This is a simplified implementation
        // A full implementation would calculate exact unchanged regions
        self.blit_full(encoder, src, dst);  // <-- ALWAYS blits everything!
    }
}
```

**Impact**: The entire "10-30x speedup" claim is invalid because it always blits the full screen

**Fix Required**: Implement proper selective blitting algorithm

### 5. Triple Buffer Texture Not Synchronized

**Location**: `wezterm-metal/src/renderer.rs:318-321`

**Issue**: Getting texture references without proper synchronization

**Current Code**:
```rust
let (current_texture, previous_texture) = {
    let buffer_lock = self.triple_buffer.as_ref().unwrap().read();
    (buffer_lock.current().clone(), buffer_lock.previous().clone())
};
```

**Problem**: Texture references are cloned (which is just a pointer copy), but the lock is immediately dropped. Another thread could swap buffers while we're using these references.

**Fix**: Hold the lock longer or use different synchronization

### 6. Blitting to Drawable Texture

**Location**: `wezterm-metal/src/renderer.rs:326-332`

**Issue**: Blitting directly to `drawable.texture()` which is the presentation target

**Current Code**:
```rust
self.blitter.as_ref().unwrap().blit_unchanged(
    &blit_encoder,
    &previous_texture,
    drawable.texture(),  // <-- Drawable texture!
    &dirty_rects,
);
```

**Problem**:
- Drawable texture should only be written by render passes, not blit encoder
- This may cause rendering artifacts or failures
- Violates Metal best practices

**Correct Approach**:
- Blit from previous to current triple buffer texture
- Then render to drawable using current triple buffer as source

### 7. Render Pass Clears Texture

**Location**: `wezterm-metal/src/renderer.rs:370`

**Issue**: Render pass has `LoadAction::Clear` which erases blitted content

**Current**: In `create_render_pass_descriptor`:
```rust
color_attachment.set_load_action(MTLLoadAction::Clear);
color_attachment.set_clear_color(MTLClearColor::new(0.0, 0.0, 0.0, 1.0));
```

**Problem**: After blitting unchanged regions, the render pass clears everything to black, destroying the blitted content!

**Fix**: Use `MTLLoadAction::Load` when incremental rendering

## 🟡 SAFETY ISSUES

### 8. Unsafe Pointer Operations

**Location**: `wezterm-metal/src/renderer.rs:346-349`

**Code**:
```rust
unsafe {
    let ptr = self.instance_buffer.contents() as *mut GlyphInstance;
    std::ptr::copy_nonoverlapping(instances.as_ptr(), ptr, instances.len());
}
```

**Issues**:
- No bounds checking
- Assumes buffer is large enough
- No alignment verification
- Could overflow if `instances.len() > max_instances`

**Fix**: Add bounds check
```rust
if instances.len() > self.max_instances {
    return Err("Too many instances".into());
}
```

### 9. Module Marked with `#![deny(unsafe_code)]`

**Location**: `wezterm-metal/src/lib.rs:11`

**Issue**: Library denies unsafe code but renderer.rs uses unsafe

**Current**:
```rust
#![deny(unsafe_code)]
```

**Problem**: The renderer module uses `unsafe` blocks for Metal buffer operations

**Fix**: Either:
- Remove `#![deny(unsafe_code)]`
- Move unsafe code to separate module
- Use safe wrappers

## 🔵 PERFORMANCE ISSUES

### 10. Always Creates Full Instance List

**Location**: `wezterm-metal/src/renderer.rs:342`

**Code**:
```rust
let instances = self.build_instances_for_rects(buffer, &dirty_rects)?;
```

**Issue**: Even when only 1 cell is dirty, builds complete instance list for dirty regions

**Impact**: For cursor blink (1 cell dirty), still does substantial CPU work

**Better**: Incrementally update instance buffer

### 11. No Dirty Tracking Integration with Terminal

**Location**: Entire codebase

**Issue**: Terminal doesn't notify renderer of changed cells

**Current**: `render_incremental()` takes `dirty_cells: &[(u16, u16)]` but nothing provides this data

**Missing**:
- Terminal write tracking
- Change notification system
- Integration between libvt and renderer

**Impact**: Renderer can't actually use incremental rendering

## 📊 VALIDATION TEST RESULTS

### Tests That Will FAIL:

1. **Compilation**: ❌ Won't compile due to signature mismatch
2. **Triple Buffer Creation**: ❌ Will panic on wrong arguments
3. **Incremental Rendering**: ❌ Always does full render
4. **Performance Claims**: ❌ No actual speedup
5. **Memory Safety**: ⚠️ Potential buffer overflows

### Tests That MIGHT Work:

1. **DirtyRect Tests**: ✅ Pure Rust logic, should pass
2. **DirtyTracker Tests**: ✅ No Metal API usage
3. **Full Screen Render**: ⚠️ May work if fallback used

## 🔧 REQUIRED FIXES

### Priority 1 (Blockers)
1. Fix TripleBuffer::new() signature
2. Add Result return type
3. Pass correct pixel format
4. Calculate texture dimensions properly

### Priority 2 (Correctness)
5. Implement actual selective blitting
6. Fix render pass load action
7. Proper triple buffer usage
8. Bounds checking in unsafe code

### Priority 3 (Integration)
9. Connect terminal change tracking to renderer
10. Proper synchronization
11. Error handling

### Priority 4 (Performance)
12. Incremental instance updates
13. Better merging algorithm
14. Profiling and validation

## 📝 TESTING REQUIREMENTS

Cannot test without:
1. macOS system
2. Metal-capable GPU
3. Fixes to compilation errors
4. Integration with terminal
5. Proper benchmarking harness

## 🎯 VERDICT

**Current Status**: 🔴 **WILL NOT COMPILE**

**Estimated Work to Fix**: 4-6 hours
- 1 hour: Fix compilation errors
- 2 hours: Implement actual blitting logic
- 1 hour: Fix synchronization issues
- 2 hours: Integration testing on macOS

**Performance Claims**: 🔴 **INVALID**
- Current implementation provides 0% speedup (always full render)
- Need to implement actual selective blitting for claims to be valid

**Recommendation**:
1. Fix compilation errors first
2. Implement stub that always does full render (safer)
3. Test basic rendering works
4. Then implement actual bit blitting
5. Benchmark to validate performance claims
