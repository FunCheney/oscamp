#![no_std]

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
pub struct EarlyAllocator<const PAGE_SIZE: usize>{
    start: usize,
    b_pos: usize,
    p_pos: usize,
    end: usize,

}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            b_pos: 0,
            p_pos: 0,
            end: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = self.end;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        Err(AllocError::NoMemory)
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let align = layout.align();
        let size = layout.size();
        // 计算对齐后的起始位置
        let alloc_start = (self.b_pos + align - 1) & !(align - 1);
        let alloc_end = alloc_start + size;
        if alloc_end > self.p_pos {
            return Err(AllocError::NoMemory);
        }
        // 更新 b_ops 的位置
        self.b_pos = alloc_end;
        Ok(unsafe { NonNull::new_unchecked(alloc_start as *mut u8) })
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        // EarlyAllocator 不支持单个释放，逻辑可以留空
    }

    fn total_bytes(&self) -> usize {
       self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        let align = 1 << align_pow2;
        let size = num_pages * PAGE_SIZE;

        // 计算对齐后的起始位置
        let alloc_end = self.p_pos;
        let alloc_start = (alloc_end - size) & !(align - 1);

        // 检查是否超出可用范围
        if alloc_start < self.b_pos || alloc_start > alloc_end {
            return Err(AllocError::NoMemory);
        }

        // 更新 p_pos 并返回分配的地址
        self.p_pos = alloc_start;
        Ok(alloc_start)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        // EarlyAllocator 不支持释放
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / PAGE_SIZE
    }
}
