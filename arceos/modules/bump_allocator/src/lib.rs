#![no_std]
extern crate alloc;

use core::alloc::Layout;
use core::ptr::NonNull;
use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,   // 起始地址
    end: usize,     // 结束地址
    b_pos: usize,   // 当前 byte 分配位置（从 start 向右增长）
    p_pos: usize,   // 当前 page 分配位置（从 end 向左增长）
    count: usize,   // 当前 byte 区域的分配计数
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = self.end;
        self.count = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        // EarlyAllocator 只支持一段连续内存，不支持多次扩展
        Err(AllocError::InvalidParam)
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let align = layout.align();
        let size = layout.size();
        let aligned_b_pos = (self.b_pos + align - 1) & !(align - 1);

        if aligned_b_pos + size <= self.p_pos {
            let ptr = aligned_b_pos;
            self.b_pos = aligned_b_pos + size;
            self.count += 1;
            // 安全：我们确保了该地址在可用区域内
            Ok(unsafe { NonNull::new_unchecked(ptr as *mut u8) })
        } else {
            Err(AllocError::MemoryOverlap)
        }
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        if self.count > 0 {
            self.count -= 1;
            if self.count == 0 {
                self.b_pos = self.start;
            }
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos.saturating_sub(self.b_pos)
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = 0;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        let total_size = num_pages * PAGE_SIZE;
        let align = 1 << align_pow2;
        let aligned = (self.p_pos - total_size) & !(align - 1);

        if self.b_pos <= aligned {
            self.p_pos = aligned;
            Ok(aligned)
        } else {
            Err(AllocError::MemoryOverlap)
        }
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        self.available_bytes() / PAGE_SIZE
    }
}
