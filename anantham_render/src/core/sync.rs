use super::device::VulkanDevice;
use ash::vk;
use std::error::Error;

pub struct SyncSetup {
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
    pub image_available: vk::Semaphore,
    pub render_finished: vk::Semaphore,
    pub in_flight: vk::Fence,
}

impl SyncSetup {
    pub fn new(vkd: &VulkanDevice) -> Result<Self, Box<dyn Error>> {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(vkd.graphics_queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = unsafe { vkd.device.create_command_pool(&pool_info, None)? };

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe { vkd.device.allocate_command_buffers(&alloc_info)?[0] };

        let sempahore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let image_available = unsafe { vkd.device.create_semaphore(&sempahore_info, None)? };
        let render_finished = unsafe { vkd.device.create_semaphore(&sempahore_info, None)? };
        let in_flight = unsafe { vkd.device.create_fence(&fence_info, None)? };

        Ok(Self {
            command_pool,
            command_buffer,
            image_available,
            render_finished,
            in_flight,
        })
    }
}
