//! Performance benchmarks for Metal renderer
//!
//! Run with: cargo bench
//!
//! Target performance (M1):
//! - Color conversion: <1ns per color
//! - Batch processing: >10M cells/sec
//! - Frame rendering: <8ms (120 FPS)

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

#[cfg(target_arch = "aarch64")]
use wezterm_metal::simd::*;

#[cfg(target_arch = "aarch64")]
fn bench_color_conversion_simd(c: &mut Criterion) {
    let mut group = c.benchmark_group("color_conversion");
    group.throughput(Throughput::Elements(1));

    group.bench_function("simd_single", |b| {
        b.iter(|| {
            let color = black_box(0xFF8040);
            color_u32_to_rgba_f32(color)
        });
    });

    group.finish();
}

#[cfg(target_arch = "aarch64")]
fn bench_batch_conversion(c: &mut Criterion) {
    let colors: Vec<u32> = (0..1000).map(|i| i * 1000).collect();
    let mut output = vec![[0.0; 4]; 1000];

    let mut group = c.benchmark_group("batch_conversion");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("batch_1000", |b| {
        b.iter(|| {
            batch_colors_to_rgba(black_box(&colors), black_box(&mut output));
        });
    });

    group.finish();
}

#[cfg(target_arch = "aarch64")]
fn bench_cell_processing(c: &mut Criterion) {
    let mut processor = CellProcessor::new(1000);
    let cells: Vec<(u32, u32)> = (0..1000).map(|i| (i * 100, i * 200)).collect();

    let mut group = c.benchmark_group("cell_processing");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("process_1000_cells", |b| {
        b.iter(|| {
            processor.process_batch(black_box(&cells), 0, 10.0, 20.0);
        });
    });

    group.finish();
}

#[cfg(not(target_arch = "aarch64"))]
fn placeholder_bench(c: &mut Criterion) {
    c.bench_function("placeholder", |b| b.iter(|| {}));
}

#[cfg(target_arch = "aarch64")]
criterion_group!(
    benches,
    bench_color_conversion_simd,
    bench_batch_conversion,
    bench_cell_processing
);

#[cfg(not(target_arch = "aarch64"))]
criterion_group!(benches, placeholder_bench);

criterion_main!(benches);
