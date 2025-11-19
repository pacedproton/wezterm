//! Memory tracking and leak detection
//!
//! Tracks all allocations and detects memory leaks

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Memory tracker for detecting leaks and monitoring usage
pub struct MemoryTracker {
    /// Total bytes allocated
    total_allocated: AtomicUsize,
    /// Total allocation count
    allocation_count: AtomicU64,
    /// Active allocations
    active_allocations: RwLock<HashMap<u64, AllocationInfo>>,
    /// Next allocation ID
    next_id: AtomicU64,
    /// Enable tracking
    enabled: bool,
}

/// Information about an allocation
#[derive(Debug, Clone)]
struct AllocationInfo {
    /// Allocation ID
    id: u64,
    /// Size in bytes
    size: usize,
    /// Allocation type
    alloc_type: AllocationType,
    /// Stack trace (simplified)
    location: String,
    /// Timestamp
    timestamp: std::time::Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationType {
    GpuTexture,
    GpuBuffer,
    AtlasEntry,
    InstanceBuffer,
    HeapAllocation,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new(enabled: bool) -> Self {
        Self {
            total_allocated: AtomicUsize::new(0),
            allocation_count: AtomicU64::new(0),
            active_allocations: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(0),
            enabled,
        }
    }

    /// Record an allocation
    pub fn allocate(
        &self,
        size: usize,
        alloc_type: AllocationType,
        location: impl Into<String>,
    ) -> AllocationId {
        if !self.enabled {
            return AllocationId(0);
        }

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.total_allocated.fetch_add(size, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        let info = AllocationInfo {
            id,
            size,
            alloc_type,
            location: location.into(),
            timestamp: std::time::Instant::now(),
        };

        self.active_allocations.write().insert(id, info);

        AllocationId(id)
    }

    /// Record a deallocation
    pub fn deallocate(&self, allocation_id: AllocationId) {
        if !self.enabled || allocation_id.0 == 0 {
            return;
        }

        if let Some(info) = self.active_allocations.write().remove(&allocation_id.0) {
            self.total_allocated.fetch_sub(info.size, Ordering::Relaxed);
        }
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        self.total_allocated.load(Ordering::Relaxed)
    }

    /// Get total allocations
    pub fn total_allocations(&self) -> u64 {
        self.allocation_count.load(Ordering::Relaxed)
    }

    /// Get active allocation count
    pub fn active_allocation_count(&self) -> usize {
        self.active_allocations.read().len()
    }

    /// Check for memory leaks
    pub fn check_leaks(&self) -> Vec<MemoryLeak> {
        if !self.enabled {
            return Vec::new();
        }

        let now = std::time::Instant::now();
        let leak_threshold = std::time::Duration::from_secs(60); // 1 minute

        self.active_allocations
            .read()
            .values()
            .filter(|info| now.duration_since(info.timestamp) > leak_threshold)
            .map(|info| MemoryLeak {
                allocation_id: info.id,
                size: info.size,
                alloc_type: info.alloc_type,
                location: info.location.clone(),
                age: now.duration_since(info.timestamp),
            })
            .collect()
    }

    /// Get memory breakdown by type
    pub fn get_breakdown(&self) -> MemoryBreakdown {
        if !self.enabled {
            return MemoryBreakdown::default();
        }

        let allocations = self.active_allocations.read();

        let mut breakdown = MemoryBreakdown::default();

        for info in allocations.values() {
            match info.alloc_type {
                AllocationType::GpuTexture => breakdown.gpu_textures += info.size,
                AllocationType::GpuBuffer => breakdown.gpu_buffers += info.size,
                AllocationType::AtlasEntry => breakdown.atlas_entries += info.size,
                AllocationType::InstanceBuffer => breakdown.instance_buffers += info.size,
                AllocationType::HeapAllocation => breakdown.heap += info.size,
            }
        }

        breakdown
    }

    /// Print memory report
    pub fn print_report(&self) {
        if !self.enabled {
            eprintln!("Memory tracking disabled");
            return;
        }

        let breakdown = self.get_breakdown();
        let total_mb = self.current_usage() as f32 / 1024.0 / 1024.0;

        eprintln!("╔═══════════════════════════════════════════╗");
        eprintln!("║     Memory Report                         ║");
        eprintln!("╠═══════════════════════════════════════════╣");
        eprintln!("║ Total Allocated:  {:<21.2}MB ║", total_mb);
        eprintln!("║ Active Allocs:    {:<23} ║", self.active_allocation_count());
        eprintln!("╠═══════════════════════════════════════════╣");
        eprintln!("║ GPU Textures:     {:<19.2}MB ║",
                 breakdown.gpu_textures as f32 / 1024.0 / 1024.0);
        eprintln!("║ GPU Buffers:      {:<19.2}MB ║",
                 breakdown.gpu_buffers as f32 / 1024.0 / 1024.0);
        eprintln!("║ Atlas Entries:    {:<19.2}MB ║",
                 breakdown.atlas_entries as f32 / 1024.0 / 1024.0);
        eprintln!("║ Instance Buffers: {:<19.2}MB ║",
                 breakdown.instance_buffers as f32 / 1024.0 / 1024.0);
        eprintln!("║ Heap:             {:<19.2}MB ║",
                 breakdown.heap as f32 / 1024.0 / 1024.0);
        eprintln!("╚═══════════════════════════════════════════╝");

        // Check for leaks
        let leaks = self.check_leaks();
        if !leaks.is_empty() {
            eprintln!("\n⚠️  Potential memory leaks detected: {}", leaks.len());
            for leak in leaks.iter().take(5) {
                eprintln!("  - {} bytes ({:?}) at {} (age: {:?})",
                         leak.size,
                         leak.alloc_type,
                         leak.location,
                         leak.age);
            }
        }
    }
}

/// Allocation ID for tracking
#[derive(Debug, Clone, Copy)]
pub struct AllocationId(u64);

/// Memory leak information
#[derive(Debug, Clone)]
pub struct MemoryLeak {
    /// Allocation ID
    pub allocation_id: u64,
    /// Size in bytes
    pub size: usize,
    /// Allocation type
    pub alloc_type: AllocationType,
    /// Location
    pub location: String,
    /// Age of allocation
    pub age: std::time::Duration,
}

/// Memory usage breakdown
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryBreakdown {
    /// GPU texture memory
    pub gpu_textures: usize,
    /// GPU buffer memory
    pub gpu_buffers: usize,
    /// Atlas entry memory
    pub atlas_entries: usize,
    /// Instance buffer memory
    pub instance_buffers: usize,
    /// Heap allocations
    pub heap: usize,
}

impl MemoryBreakdown {
    /// Total memory
    pub fn total(&self) -> usize {
        self.gpu_textures
            + self.gpu_buffers
            + self.atlas_entries
            + self.instance_buffers
            + self.heap
    }
}

/// RAII guard for tracked allocations
pub struct TrackedAllocation<'a> {
    tracker: &'a MemoryTracker,
    id: AllocationId,
}

impl<'a> TrackedAllocation<'a> {
    /// Create a new tracked allocation
    pub fn new(
        tracker: &'a MemoryTracker,
        size: usize,
        alloc_type: AllocationType,
        location: impl Into<String>,
    ) -> Self {
        let id = tracker.allocate(size, alloc_type, location);
        Self { tracker, id }
    }
}

impl<'a> Drop for TrackedAllocation<'a> {
    fn drop(&mut self) {
        self.tracker.deallocate(self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracking() {
        let tracker = MemoryTracker::new(true);

        let id1 = tracker.allocate(1024, AllocationType::GpuBuffer, "test1");
        assert_eq!(tracker.current_usage(), 1024);
        assert_eq!(tracker.active_allocation_count(), 1);

        let id2 = tracker.allocate(2048, AllocationType::GpuTexture, "test2");
        assert_eq!(tracker.current_usage(), 3072);
        assert_eq!(tracker.active_allocation_count(), 2);

        tracker.deallocate(id1);
        assert_eq!(tracker.current_usage(), 2048);
        assert_eq!(tracker.active_allocation_count(), 1);

        tracker.deallocate(id2);
        assert_eq!(tracker.current_usage(), 0);
        assert_eq!(tracker.active_allocation_count(), 0);
    }

    #[test]
    fn test_memory_breakdown() {
        let tracker = MemoryTracker::new(true);

        tracker.allocate(1000, AllocationType::GpuTexture, "tex");
        tracker.allocate(2000, AllocationType::GpuBuffer, "buf");
        tracker.allocate(500, AllocationType::AtlasEntry, "atlas");

        let breakdown = tracker.get_breakdown();
        assert_eq!(breakdown.gpu_textures, 1000);
        assert_eq!(breakdown.gpu_buffers, 2000);
        assert_eq!(breakdown.atlas_entries, 500);
        assert_eq!(breakdown.total(), 3500);
    }

    #[test]
    fn test_tracked_allocation_raii() {
        let tracker = MemoryTracker::new(true);

        {
            let _alloc = TrackedAllocation::new(&tracker, 512, AllocationType::HeapAllocation, "raii_test");
            assert_eq!(tracker.current_usage(), 512);
        }

        // Should be deallocated
        assert_eq!(tracker.current_usage(), 0);
    }
}
