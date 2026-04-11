use ash::{Instance, vk};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use std::error::Error;

pub struct GpuAllocator {
    pub inner: std::mem::ManuallyDrop<Allocator>,
}

impl GpuAllocator {
    pub fn new(
        instance: Instance,
        device: ash::Device,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self, Box<dyn Error>> {
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance,
            device,
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address: false,
            allocation_sizes: Default::default(),
        })?;

        Ok(Self {
            inner: std::mem::ManuallyDrop::new(allocator),
        })
    }
}

impl Drop for GpuAllocator {
    fn drop(&mut self) {
        unsafe {
            std::mem::ManuallyDrop::drop(&mut self.inner);
        }
    }
}
