use crate::profiler::AllocationProfiler;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    static IN_ALLOCATOR: Cell<bool> = Cell::new(false);
}

pub struct ProfilingAllocator;

unsafe impl GlobalAlloc for ProfilingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: System is the standard allocator
        let ptr = unsafe { System.alloc(layout) };

        if !ptr.is_null() {
            // Check for reentrancy immediately
            let should_profile = IN_ALLOCATOR.with(|flag| {
                if flag.get() {
                    false // Already in allocator, skip profiling
                } else {
                    flag.set(true);
                    true
                }
            });

            if should_profile {
                let backtrace = backtrace::Backtrace::new_unresolved();
                AllocationProfiler::record_allocation(layout.size(), backtrace);
                IN_ALLOCATOR.with(|flag| flag.set(false));
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Only record deallocations when not in a reentrant call
        let should_profile = IN_ALLOCATOR.with(|flag| !flag.get());
        if should_profile {
            AllocationProfiler::record_deallocation(layout.size());
        }
        // SAFETY: System is the standard allocator, ptr/layout come from alloc
        unsafe { System.dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: System is the standard allocator, ptr/layout come from alloc
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };

        if !new_ptr.is_null() {
            // Check for reentrancy
            let should_profile = IN_ALLOCATOR.with(|flag| {
                if flag.get() {
                    false
                } else {
                    flag.set(true);
                    true
                }
            });

            if should_profile {
                // Record deallocation of old size and allocation of new size
                AllocationProfiler::record_deallocation(layout.size());
                let backtrace = backtrace::Backtrace::new_unresolved();
                AllocationProfiler::record_allocation(new_size, backtrace);
                IN_ALLOCATOR.with(|flag| flag.set(false));
            }
        }

        new_ptr
    }
}
#[global_allocator]
static GLOBAL: ProfilingAllocator = ProfilingAllocator;
