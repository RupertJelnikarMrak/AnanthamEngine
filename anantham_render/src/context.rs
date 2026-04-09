use ash::{Entry, Instance, ext::mesh_shader, vk};
use bevy_ecs::prelude::Resource;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::error::Error;
use winit::window::Window;

#[derive(Resource)]
pub struct VulkanContext {
    pub entry: Entry,
    pub instance: Instance,
    pub surface_ext: ash::khr::surface::Instance,
    pub surface: vk::SurfaceKHR,

    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub graphics_queue: vk::Queue,
    pub graphics_queue_family_index: u32,
    pub mesh_ext: mesh_shader::Device,

    pub swapchain_ext: ash::khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,

    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,

    pub pipeline_layout: vk::PipelineLayout,
    pub graphics_pipeline: vk::Pipeline,
}

impl VulkanContext {
    pub fn new(window: &Window) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe { Entry::load()? };
        let display_handle = window.display_handle()?.as_raw();
        let window_handle = window.window_handle()?.as_raw();
        let required_extensions = ash_window::enumerate_required_extensions(display_handle)?;

        let app_info = vk::ApplicationInfo::default().api_version(vk::make_api_version(0, 1, 4, 0));
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(required_extensions);

        let instance = unsafe { entry.create_instance(&create_info, None)? };
        let surface = unsafe {
            ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)?
        };
        let surface_ext = ash::khr::surface::Instance::new(&entry, &instance);

        let pdevices = unsafe { instance.enumerate_physical_devices()? };
        let (physical_device, graphics_queue_family_index) = pdevices
            .into_iter()
            .filter_map(|pdevice| {
                let queue_families =
                    unsafe { instance.get_physical_device_queue_family_properties(pdevice) };
                let queue_index = queue_families.iter().enumerate().position(|(i, info)| {
                    let supports_graphics = info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                    let supports_surface = unsafe {
                        surface_ext
                            .get_physical_device_surface_support(pdevice, i as u32, surface)
                            .unwrap_or(false)
                    };
                    supports_graphics && supports_surface
                });
                queue_index.map(|index| (1000, pdevice, index as u32)) // Simplified for this snippet
            })
            .max_by_key(|&(score, _, _)| score)
            .map(|(_, pdevice, index)| (pdevice, index))
            .unwrap();

        let queue_priorities = [1.0];
        let queue_create_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_queue_family_index)
            .queue_priorities(&queue_priorities);

        let device_extensions = [
            ash::khr::swapchain::NAME.as_ptr(),
            mesh_shader::NAME.as_ptr(),
        ];
        let mut vulkan_13_features =
            vk::PhysicalDeviceVulkan13Features::default().dynamic_rendering(true);
        let mut mesh_features = vk::PhysicalDeviceMeshShaderFeaturesEXT::default()
            .mesh_shader(true)
            .task_shader(false);
        let mut features2 = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut vulkan_13_features)
            .push_next(&mut mesh_features);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_create_info))
            .enabled_extension_names(&device_extensions)
            .push_next(&mut features2);

        let device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };
        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family_index, 0) };
        let mesh_ext = mesh_shader::Device::new(&instance, &device);

        // ## Swapchain ##
        let capabilities = unsafe {
            surface_ext.get_physical_device_surface_capabilities(physical_device, surface)?
        };
        let formats =
            unsafe { surface_ext.get_physical_device_surface_formats(physical_device, surface)? };
        let format = formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&formats[0]);

        let extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            let size = window.inner_size();
            vk::Extent2D {
                width: size.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: size.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        };

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(capabilities.min_image_count + 1)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain_ext = ash::khr::swapchain::Device::new(&instance, &device);
        let swapchain = unsafe { swapchain_ext.create_swapchain(&create_info, None)? };
        let swapchain_images = unsafe { swapchain_ext.get_swapchain_images(swapchain)? };

        let swapchain_image_views: Vec<vk::ImageView> = swapchain_images
            .iter()
            .map(|&image| {
                let view_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format.format)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                unsafe { device.create_image_view(&view_info, None).unwrap() }
            })
            .collect();

        // ## Commands and Sync ##
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(graphics_queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = unsafe { device.create_command_pool(&pool_info, None)? };
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info)?[0] };

        let sempahore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let image_available_semaphore = unsafe { device.create_semaphore(&sempahore_info, None)? };
        let render_finished_semaphore = unsafe { device.create_semaphore(&sempahore_info, None)? };
        let in_flight_fence = unsafe { device.create_fence(&fence_info, None)? };

        // ## Pipeline ##
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

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE);
        let multisample_info = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA);
        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let layout_info = vk::PipelineLayoutCreateInfo::default();
        let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None)? };

        let color_attachment_formats = [format.format];
        let mut pipeline_rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_attachment_formats);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_info)
            .color_blend_state(&color_blend_info)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .push_next(&mut pipeline_rendering_info);

        let graphics_pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .unwrap()[0]
        };

        unsafe {
            device.destroy_shader_module(mesh_module, None);
            device.destroy_shader_module(frag_module, None);
        }

        tracing::info!("Vulkan Context and Mesh Pipeline Created successfully.");

        Ok(Self {
            entry,
            instance,
            surface_ext,
            surface,
            physical_device,
            device,
            graphics_queue,
            graphics_queue_family_index,
            mesh_ext,
            swapchain_ext,
            swapchain,
            swapchain_images,
            swapchain_image_views,
            swapchain_format: format.format,
            swapchain_extent: extent,
            command_pool,
            command_buffer,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
            pipeline_layout,
            graphics_pipeline,
        })
    }

    pub fn draw_frame(&mut self) -> Result<(), Box<dyn Error>> {
        let device = &self.device;
        let swapchain_ext = &self.swapchain_ext;

        unsafe {
            device.wait_for_fences(&[self.in_flight_fence], true, u64::MAX)?;
            device.reset_fences(&[self.in_flight_fence])?;

            let (image_index, _) = swapchain_ext.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null(),
            )?;

            device
                .reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())?;

            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device.begin_command_buffer(self.command_buffer, &begin_info)?;

            let image = self.swapchain_images[image_index as usize];
            let image_view = self.swapchain_image_views[image_index as usize];

            let mut image_memory_barrier = vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&image_memory_barrier),
            );

            let clear_value = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.15, 1.0],
                },
            };
            let color_attachment_info = vk::RenderingAttachmentInfo::default()
                .image_view(image_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(clear_value);

            let rendering_info = vk::RenderingInfo::default()
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain_extent,
                })
                .layer_count(1)
                .color_attachments(std::slice::from_ref(&color_attachment_info));

            device.cmd_begin_rendering(self.command_buffer, &rendering_info);
            device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline,
            );

            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.swapchain_extent.width as f32,
                height: self.swapchain_extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            device.cmd_set_viewport(self.command_buffer, 0, &[viewport]);

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            };
            device.cmd_set_scissor(self.command_buffer, 0, &[scissor]);

            self.mesh_ext
                .cmd_draw_mesh_tasks(self.command_buffer, 1, 1, 1);

            device.cmd_end_rendering(self.command_buffer);

            image_memory_barrier.src_access_mask = vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
            image_memory_barrier.dst_access_mask = vk::AccessFlags::empty();
            image_memory_barrier.old_layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
            image_memory_barrier.new_layout = vk::ImageLayout::PRESENT_SRC_KHR;

            device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&image_memory_barrier),
            );

            device.end_command_buffer(self.command_buffer)?;

            let wait_semaphores = [self.image_available_semaphore];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.command_buffer];
            let signal_semaphores = [self.render_finished_semaphore];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            device.queue_submit(self.graphics_queue, &[submit_info], self.in_flight_fence)?;

            let swapchains = [self.swapchain];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            swapchain_ext.queue_present(self.graphics_queue, &present_info)?;
        }
        Ok(())
    }
}
