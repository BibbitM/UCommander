//! Package summary: a damaged count does not cause an allocation larger
//! than the input. This test binary has its own allocator, which records
//! the largest allocation.

mod common;

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use common::{Str, Summary};
use uc_format::package::read_summary;

/// The system allocator, with a record of the largest allocation size.
struct Recording;

static LARGEST: AtomicUsize = AtomicUsize::new(0);

// SAFETY: Each function forwards the call to the system allocator with the
// same arguments, so the `GlobalAlloc` contract of `System` applies.
unsafe impl GlobalAlloc for Recording {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LARGEST.fetch_max(layout.size(), Ordering::Relaxed);
        // SAFETY: The caller fulfills the contract of `GlobalAlloc::alloc`.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` came from `System.alloc` with this `layout`.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        LARGEST.fetch_max(new_size, Ordering::Relaxed);
        // SAFETY: The caller fulfills the contract of `GlobalAlloc::realloc`.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Recording = Recording;

#[test]
fn a_damaged_additional_package_count_allocates_no_more_than_the_input() {
    // The count fits in the remaining input (4 bytes for each string), but
    // the first string length is damaged.
    const COUNT: usize = 1_000_000;
    let mut bytes = Summary {
        additional_packages_to_cook: vec![Str::Length(i32::MAX)],
        additional_package_count: Some(COUNT as i32),
        ..Summary::default()
    }
    .to_bytes();
    bytes.resize(bytes.len() + COUNT * 4, 0);
    bytes.shrink_to_fit();
    LARGEST.store(0, Ordering::Relaxed);

    assert!(read_summary(&bytes).is_err());

    let largest = LARGEST.load(Ordering::Relaxed);
    assert!(
        largest <= bytes.len(),
        "largest allocation {largest}, input {}",
        bytes.len()
    );
}
