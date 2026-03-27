use std::alloc::{Allocator, Layout, AllocError};
use std::ptr::NonNull;
use std::cell::Cell;

pub struct CityAllocator {
    arena: NonNull<u8>,
    next: Cell<NonNull<u8>>,
}

// Max size (in bytes) to store all the city names.
const MAX_SIZE: usize = 100 * 10_000;

impl CityAllocator {
    pub fn new() -> Self {
        let buffer = Box::into_raw(Box::new([0u8; MAX_SIZE]));

        let ptr = NonNull::new(buffer.cast::<u8>()).unwrap();

        Self {
            arena: ptr,
            next: Cell::new(ptr),
        }
    }
}

impl Drop for CityAllocator {
    fn drop(&mut self) {
        let mut ptr = self.arena.cast::<[u8; MAX_SIZE]>();

        // Deallocate the box.
        _ = unsafe { Box::from_raw(ptr.as_mut()) };
    }
}

unsafe impl Allocator for CityAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if layout.size() > 100 {
            return Err(AllocError);
        }

        if self.next.get().addr().get() + layout.size() >
            self.arena.addr().get() + MAX_SIZE {
            return Err(AllocError);
        }

        let ptr = self.next.get();

        // bump the next pointer.
        self.next.set(unsafe { ptr.add(layout.size()) });

        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // nop
    }
}
