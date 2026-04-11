use crate::context::MeshPushConstants;
use ash::vk;
use std::error::Error; // Assuming this struct remains in context.rs for now

pub struct GraphicsPipelines {
    pub layout: vk::PipelineLayout,
    pub opaque: vk::Pipeline,
    pub transparent: vk::Pipeline,
}

impl GraphicsPipelines {
    pub fn new(
        device: &ash::Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
        color_format: vk::Format,
        depth_format: vk::Format,
    ) -> Result<Self, Box<dyn Error>> {
        let mesh_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/shader.mesh.spv"));
        let mesh_code = ash::util::read_spv(&mut std::io::Cursor::new(&mesh_bytes[..]))?;
        let mesh_info = vk::ShaderModuleCreateInfo::default().code(&mesh_code);
        let mesh_module = unsafe { device.create_shader_module(&mesh_info, None)? };

        let frag_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/shader.frag.spv"));
        let frag_code = ash::util::read_spv(&mut std::io::Cursor::new(&frag_bytes[..]))?;
        let frag_info = vk::ShaderModuleCreateInfo::default().code(&frag_code);
        let frag_module = unsafe { device.create_shader_module(&frag_info, None)? };

        let entry_name = c"main";
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::MESH_EXT)
                .module(mesh_module)
                .name(entry_name),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_module)
                .name(entry_name),
        ];

        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::MESH_EXT)
            .offset(0)
            .size(std::mem::size_of::<MeshPushConstants>() as u32);

        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout))
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));
        let layout = unsafe { device.create_pipeline_layout(&layout_info, None)? };

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let multisample_info = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_attachment_formats = [color_format];
        let mut pipeline_rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_attachment_formats)
            .depth_attachment_format(depth_format);

        // --- OPAQUE PIPELINE ---
        let opaque_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);
        let opaque_blend_info = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(std::slice::from_ref(&opaque_blend_attachment));

        let opaque_depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        let opaque_rasterization_info = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

        let opaque_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&opaque_rasterization_info)
            .multisample_state(&multisample_info)
            .color_blend_state(&opaque_blend_info)
            .depth_stencil_state(&opaque_depth_stencil_info)
            .dynamic_state(&dynamic_state_info)
            .layout(layout)
            .push_next(&mut pipeline_rendering_info);

        let opaque = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[opaque_pipeline_info], None)
                .unwrap()[0]
        };

        // --- TRANSPARENT PIPELINE ---
        let transparent_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);
        let transparent_blend_info = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(std::slice::from_ref(&transparent_blend_attachment));

        let transparent_depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        let transparent_rasterization_info = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

        let transparent_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&transparent_rasterization_info)
            .multisample_state(&multisample_info)
            .color_blend_state(&transparent_blend_info)
            .depth_stencil_state(&transparent_depth_stencil_info)
            .dynamic_state(&dynamic_state_info)
            .layout(layout)
            .push_next(&mut pipeline_rendering_info);

        let transparent = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[transparent_pipeline_info],
                    None,
                )
                .unwrap()[0]
        };

        unsafe {
            device.destroy_shader_module(mesh_module, None);
            device.destroy_shader_module(frag_module, None);
        }

        tracing::info!("Mesh Shader Pipelines compiled successfully.");

        Ok(Self {
            layout,
            opaque,
            transparent,
        })
    }
}
