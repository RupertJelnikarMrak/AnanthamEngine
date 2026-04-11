use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use std::error::Error;

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub offset: u32,
    pub size: u32,
}

pub struct GeometryArena {
    pub buffer: vk::Buffer,
    pub allocation: Option<Allocation>,
    pub capacity: u32,
    pub free_blocks: Vec<Block>,
}

impl GeometryArena {
    pub fn new(device: &ash::Device, allocator: &mut Allocator) -> Result<Self, Box<dyn Error>> {
        let capacity = 256 * 1024 * 1024; // 256 MB Arena
        let buffer_info = vk::BufferCreateInfo::default()
            .size(capacity as u64)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { device.create_buffer(&buffer_info, None)? };

        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let allocation = allocator.allocate(&AllocationCreateDesc {
            name: "Geometry Arena",
            requirements,
            location: MemoryLocation::CpuToGpu, // CPU writes, GPU reads
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        unsafe {
            device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())?;
        };

        // Initially, the entire arena is one giant free block
        let free_blocks = vec![Block {
            offset: 0,
            size: capacity,
        }];

        Ok(Self {
            buffer,
            allocation: Some(allocation),
            capacity,
            free_blocks,
        })
    }

    /// Allocates a sub-region within the Vulkan buffer.
    /// Returns the byte offset if successful, or None if out of memory/fragmented.
    pub fn allocate(&mut self, mut size: u32) -> Option<u32> {
        // Align allocations to 256 bytes to prevent micro-fragmentation
        let alignment = 256;
        size = (size + alignment - 1) & !(alignment - 1);

        for i in 0..self.free_blocks.len() {
            if self.free_blocks[i].size >= size {
                let offset = self.free_blocks[i].offset;

                if self.free_blocks[i].size == size {
                    self.free_blocks.remove(i);
                } else {
                    self.free_blocks[i].offset += size;
                    self.free_blocks[i].size -= size;
                }
                return Some(offset);
            }
        }

        tracing::error!("Geometry Arena is out of memory or too fragmented!");
        None
    }

    /// Returns a block of memory back to the arena and coalesces adjacent free blocks.
    pub fn free(&mut self, offset: u32, mut size: u32) {
        let alignment = 256;
        size = (size + alignment - 1) & !(alignment - 1);

        self.free_blocks.push(Block { offset, size });

        self.free_blocks.sort_unstable_by_key(|b| b.offset);

        let mut i = 0;
        while i < self.free_blocks.len() - 1 {
            let current = self.free_blocks[i];
            let next = self.free_blocks[i + 1];

            if current.offset + current.size == next.offset {
                self.free_blocks[i].size += next.size;
                self.free_blocks.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }

    /// Writes raw byte data directly to the mapped GPU memory at the specified offset.
    pub fn upload(&self, offset: u32, data: &[u8]) {
        if let Some(alloc) = &self.allocation
            && let Some(mapped_ptr) = alloc.mapped_ptr()
        {
            unsafe {
                let dst = mapped_ptr.as_ptr().add(offset as usize);
                std::ptr::copy_nonoverlapping(data.as_ptr(), dst as *mut u8, data.len());
            }
        }
    }

    pub fn destroy(&mut self, device: &ash::Device, allocator: &mut Allocator) {
        if let Some(alloc) = self.allocation.take() {
            allocator.free(alloc).unwrap();
        }
        unsafe {
            device.destroy_buffer(self.buffer, None);
        }
    }
}
