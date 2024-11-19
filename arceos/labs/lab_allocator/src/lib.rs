//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]

use allocator::{BaseAllocator, ByteAllocator, AllocResult, AllocError};
use core::ptr::NonNull;
use core::alloc::Layout;

// 内存块表示
#[derive(Copy, Clone)]
struct MemoryBlock {
    start: usize, // 内存块的起始地址
    size: usize,  // 内存块的大小
    in_use: bool, // 是否正在使用
}

// 内存分配器实现
pub struct LabByteAllocator {
    memory_pool_start: usize,    // 内存池的起始地址
    memory_pool_size: usize,     // 内存池的总大小
    blocks: [Option<MemoryBlock>; 1024], // 内存块的管理数组（固定大小）
    total_used: usize,           // 已用字节数
}

impl LabByteAllocator {
    pub const fn new() -> Self {
        Self {
            memory_pool_start: 0,
            memory_pool_size: 0,
            blocks: [None; 1024],
            total_used: 0,
        }
    }
}


impl BaseAllocator for LabByteAllocator {
    // 初始化内存池
    fn init(&mut self, start: usize, size: usize) {
        self.memory_pool_start = start;
        self.memory_pool_size = size;
        self.blocks[0] = Some(MemoryBlock {
            start,
            size,
            in_use: false,
        });
    }

    // 添加新的内存区域
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        for block in self.blocks.iter_mut() {
            if block.is_none() {
                *block = Some(MemoryBlock {
                    start,
                    size,
                    in_use: false,
                });
                return Ok(());
            }
        }
        Err(AllocError::NoMemory)
    }
}


impl ByteAllocator for LabByteAllocator {
    // 分配内存块
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();
        let mut new_block = None; // 用于存储新的内存块信息

        for block in self.blocks.iter_mut() {
            if let Some(b) = block {
                if !b.in_use && b.size >= size {
                    // 计算对齐后的起始地址和分配结束地址
                    let aligned_start = (b.start + align - 1) & !(align - 1);
                    let end = aligned_start + size;

                    if end <= b.start + b.size {
                        b.in_use = true;
                        self.total_used += size;

                        // 如果有剩余空间，记录新块的信息
                        if end < b.start + b.size {
                            new_block = Some((end, b.start + b.size - end));
                        }

                        b.size = size;
                        return Ok(NonNull::new(aligned_start as *mut u8).unwrap());
                    }
                }
            }
        }
        Err(AllocError::NoMemory)
    }

    // 释放内存块
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        let addr = pos.as_ptr() as usize;
        let size = layout.size();

        for block in self.blocks.iter_mut() {
            if let Some(b) = block {
                if b.start == addr && b.in_use {
                    b.in_use = false;
                    self.total_used -= size;
                    return;
                }
            }
        }
    }

    // 总字节数
    fn total_bytes(&self) -> usize {
        self.memory_pool_size
    }

    // 已用字节数
    fn used_bytes(&self) -> usize {
        self.total_used
    }

    // 可用字节数
    fn available_bytes(&self) -> usize {
        self.memory_pool_size - self.total_used
    }
}



