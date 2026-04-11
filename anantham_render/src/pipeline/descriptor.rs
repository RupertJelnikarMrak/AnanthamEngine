use ash::vk;
use std::error::Error;

pub struct DescriptorSetup {
    pub pool: vk::DescriptorPool,
    pub layout: vk::DescriptorSetLayout,
    pub set: vk::DescriptorSet,
}

impl DescriptorSetup {
    pub fn new(device: &ash::Device, vertex_buffer: vk::Buffer) -> Result<Self, Box<dyn Error>> {
        let binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::MESH_EXT);

        let layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(std::slice::from_ref(&binding));
        let layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let pool_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1);
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(std::slice::from_ref(&pool_size))
            .max_sets(1);
        let pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(std::slice::from_ref(&layout));
        let set = unsafe { device.allocate_descriptor_sets(&alloc_info)?[0] };

        let desc_buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(vertex_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);

        let write = vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&desc_buffer_info));

        unsafe { device.update_descriptor_sets(std::slice::from_ref(&write), &[]) };

        Ok(Self { pool, layout, set })
    }
}
