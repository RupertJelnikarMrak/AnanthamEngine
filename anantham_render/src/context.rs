use anantham_core::render_bridge::components::{ExtractedMeshes, ExtractedView, Vertex};
use ash::{Entry, Instance, ext::mesh_shader, vk};
use bevy_ecs::prelude::Resource;
use glam::Mat4;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{
    Allocation, AllocationCreateDesc, AllocationScheme, Allocator, AllocatorCreateDesc,
};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::error::Error;
use std::ffi::CStr;
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
    pub transparent_pipeline: vk::Pipeline,

    pub allocator: std::mem::ManuallyDrop<Allocator>,
    pub vertex_buffer: vk::Buffer,
    pub vertex_allocation: Option<Allocation>,

    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_set: vk::DescriptorSet,

    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_allocation: Option<Allocation>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshPushConstants {
    pub mvp: glam::Mat4,
    pub vertex_offset: u32,
    pub _padding: [u32; 3],
}

struct SwapchainSetup {
    ext: ash::khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    format: vk::Format,
    extent: vk::Extent2D,
}

struct SyncSetup {
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    image_available: vk::Semaphore,
    render_finished: vk::Semaphore,
    in_flight: vk::Fence,
}

struct FrameGeometry {
    vertices: Vec<Vertex>,
    opaque_draws: Vec<(Mat4, u32, u32)>,
    transparent_draws: Vec<(Mat4, u32, u32)>,
}

// ============================================================================
// INITIALIZATION HELPERS
// ============================================================================
impl VulkanContext {
    pub fn new(window: &Window) -> Result<Self, Box<dyn Error>> {
        tracing::debug!("Starting Vulkan boot sequence...");

        let entry = unsafe { Entry::load()? };
        let (instance, surface_ext, surface) = Self::create_instance_and_surface(&entry, window)?;

        let (physical_device, graphics_queue_family_index) =
            Self::pick_physical_device(&instance, &surface_ext, surface)?;

        let (device, graphics_queue, mesh_ext) =
            Self::create_logical_device(&instance, physical_device, graphics_queue_family_index)?;

        let swap_setup = Self::create_swapchain(
            window,
            &instance,
            &device,
            physical_device,
            surface,
            &surface_ext,
        )?;

        let sync_setup = Self::create_commands_and_sync(&device, graphics_queue_family_index)?;

        let mut allocator =
            Self::create_allocator(instance.clone(), device.clone(), physical_device)?;

        let (vertex_buffer, vertex_allocation) =
            Self::create_vertex_buffer(&device, &mut allocator)?;

        let depth_format = vk::Format::D32_SFLOAT;
        let (depth_image, depth_image_view, depth_allocation) =
            Self::create_depth_buffer(&device, &mut allocator, swap_setup.extent, depth_format)?;

        let (descriptor_pool, descriptor_set_layout, descriptor_set) =
            Self::create_descriptors(&device, vertex_buffer)?;

        let (pipeline_layout, graphics_pipeline, transparent_pipeline) = Self::create_pipelines(
            &device,
            descriptor_set_layout,
            swap_setup.format,
            depth_format,
        )?;

        tracing::info!("Vulkan Context fully initialized");

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
            swapchain_ext: swap_setup.ext,
            swapchain: swap_setup.swapchain,
            swapchain_images: swap_setup.images,
            swapchain_image_views: swap_setup.image_views,
            swapchain_format: swap_setup.format,
            swapchain_extent: swap_setup.extent,
            command_pool: sync_setup.command_pool,
            command_buffer: sync_setup.command_buffer,
            image_available_semaphore: sync_setup.image_available,
            render_finished_semaphore: sync_setup.render_finished,
            in_flight_fence: sync_setup.in_flight,
            pipeline_layout,
            graphics_pipeline,
            transparent_pipeline,
            allocator: std::mem::ManuallyDrop::new(allocator),
            vertex_buffer,
            vertex_allocation: Some(vertex_allocation),
            descriptor_pool,
            descriptor_set_layout,
            descriptor_set,
            depth_image,
            depth_image_view,
            depth_allocation: Some(depth_allocation),
        })
    }

    fn create_instance_and_surface(
        entry: &Entry,
        window: &Window,
    ) -> Result<(Instance, ash::khr::surface::Instance, vk::SurfaceKHR), Box<dyn Error>> {
        let display_handle = window.display_handle()?.as_raw();
        let window_handle = window.window_handle()?.as_raw();
        let required_extensions = ash_window::enumerate_required_extensions(display_handle)?;

        let app_info = vk::ApplicationInfo::default().api_version(vk::make_api_version(0, 1, 4, 0));
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(required_extensions);

        let instance = unsafe { entry.create_instance(&create_info, None)? };
        let surface = unsafe {
            ash_window::create_surface(entry, &instance, display_handle, window_handle, None)?
        };
        let surface_ext = ash::khr::surface::Instance::new(entry, &instance);

        Ok((instance, surface_ext, surface))
    }

    fn pick_physical_device(
        instance: &Instance,
        surface_ext: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> Result<(vk::PhysicalDevice, u32), Box<dyn Error>> {
        let pdevices = unsafe { instance.enumerate_physical_devices()? };
        let (physical_device, queue_family_index) = pdevices
            .into_iter()
            .filter_map(|pdevice| {
                let properties = unsafe { instance.get_physical_device_properties(pdevice) };
                let device_type = properties.device_type;

                let available_extensions = unsafe {
                    instance
                        .enumerate_device_extension_properties(pdevice)
                        .unwrap_or_default()
                };

                let has_mesh_shader = available_extensions.iter().any(|ext| unsafe {
                    CStr::from_ptr(ext.extension_name.as_ptr()) == ash::ext::mesh_shader::NAME
                });
                let has_swapchain = available_extensions.iter().any(|ext| unsafe {
                    CStr::from_ptr(ext.extension_name.as_ptr()) == ash::khr::swapchain::NAME
                });

                if !has_mesh_shader || !has_swapchain {
                    return None;
                }

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

                if let Some(index) = queue_index {
                    let mut score = 0;
                    if device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                        score += 1000;
                    } else if device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
                        score += 100;
                    }
                    Some((score, pdevice, index as u32))
                } else {
                    None
                }
            })
            .max_by_key(|&(score, _, _)| score)
            .map(|(_, pdevice, index)| (pdevice, index))
            .expect("Failed to find a suitable physical device with Mesh Shader support");

        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) };
        tracing::info!("Selected Physical Device: {:?}", device_name);

        Ok((physical_device, queue_family_index))
    }

    fn create_logical_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<(ash::Device, vk::Queue, mesh_shader::Device), Box<dyn Error>> {
        let queue_priorities = [1.0];
        let queue_create_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
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
        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        let mesh_ext = mesh_shader::Device::new(instance, &device);

        Ok((device, graphics_queue, mesh_ext))
    }

    fn create_swapchain(
        window: &Window,
        instance: &Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_ext: &ash::khr::surface::Instance,
    ) -> Result<SwapchainSetup, Box<dyn Error>> {
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
            .present_mode(vk::PresentModeKHR::IMMEDIATE)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain_ext = ash::khr::swapchain::Device::new(instance, device);
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

        Ok(SwapchainSetup {
            ext: swapchain_ext,
            swapchain,
            images: swapchain_images,
            image_views: swapchain_image_views,
            format: format.format,
            extent,
        })
    }

    fn create_commands_and_sync(
        device: &ash::Device,
        queue_family_index: u32,
    ) -> Result<SyncSetup, Box<dyn Error>> {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
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

        Ok(SyncSetup {
            command_pool,
            command_buffer,
            image_available: image_available_semaphore,
            render_finished: render_finished_semaphore,
            in_flight: in_flight_fence,
        })
    }

    fn create_allocator(
        instance: Instance,
        device: ash::Device,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Allocator, Box<dyn Error>> {
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance,
            device,
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address: false,
            allocation_sizes: Default::default(),
        })?;
        Ok(allocator)
    }

    fn create_vertex_buffer(
        device: &ash::Device,
        allocator: &mut Allocator,
    ) -> Result<(vk::Buffer, Allocation), Box<dyn Error>> {
        let arena_size = 256 * 1024 * 1024;
        let buffer_info = vk::BufferCreateInfo::default()
            .size(arena_size as u64)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let vertex_buffer = unsafe { device.create_buffer(&buffer_info, None)? };

        let requirements = unsafe { device.get_buffer_memory_requirements(vertex_buffer) };
        let vertex_allocation = allocator.allocate(&AllocationCreateDesc {
            name: "Geometry Arena",
            requirements,
            location: MemoryLocation::CpuToGpu,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        unsafe {
            device.bind_buffer_memory(
                vertex_buffer,
                vertex_allocation.memory(),
                vertex_allocation.offset(),
            )?
        };

        Ok((vertex_buffer, vertex_allocation))
    }

    fn create_depth_buffer(
        device: &ash::Device,
        allocator: &mut Allocator,
        extent: vk::Extent2D,
        depth_format: vk::Format,
    ) -> Result<(vk::Image, vk::ImageView, Allocation), Box<dyn Error>> {
        let depth_image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(depth_format)
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let depth_image = unsafe { device.create_image(&depth_image_info, None)? };
        let depth_reqs = unsafe { device.get_image_memory_requirements(depth_image) };
        let depth_allocation = allocator.allocate(&AllocationCreateDesc {
            name: "Depth Buffer",
            requirements: depth_reqs,
            location: MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        unsafe {
            device.bind_image_memory(
                depth_image,
                depth_allocation.memory(),
                depth_allocation.offset(),
            )?;
        }

        let depth_view_info = vk::ImageViewCreateInfo::default()
            .image(depth_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(depth_format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        let depth_image_view = unsafe { device.create_image_view(&depth_view_info, None)? };

        Ok((depth_image, depth_image_view, depth_allocation))
    }

    fn create_descriptors(
        device: &ash::Device,
        vertex_buffer: vk::Buffer,
    ) -> Result<
        (
            vk::DescriptorPool,
            vk::DescriptorSetLayout,
            vk::DescriptorSet,
        ),
        Box<dyn Error>,
    > {
        let binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::MESH_EXT);

        let layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(std::slice::from_ref(&binding));
        let descriptor_set_layout =
            unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let pool_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1);
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(std::slice::from_ref(&pool_size))
            .max_sets(1);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&descriptor_set_layout));
        let descriptor_set = unsafe { device.allocate_descriptor_sets(&alloc_info)?[0] };

        let desc_buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(vertex_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);

        let write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&desc_buffer_info));

        unsafe { device.update_descriptor_sets(std::slice::from_ref(&write), &[]) };

        Ok((descriptor_pool, descriptor_set_layout, descriptor_set))
    }

    fn create_pipelines(
        device: &ash::Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
        color_format: vk::Format,
        depth_format: vk::Format,
    ) -> Result<(vk::PipelineLayout, vk::Pipeline, vk::Pipeline), Box<dyn Error>> {
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
        let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None)? };

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
            .layout(pipeline_layout)
            .push_next(&mut pipeline_rendering_info);

        let graphics_pipeline = unsafe {
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

        let mut transparent_rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_attachment_formats)
            .depth_attachment_format(depth_format);

        let transparent_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&transparent_rasterization_info)
            .multisample_state(&multisample_info)
            .color_blend_state(&transparent_blend_info)
            .depth_stencil_state(&transparent_depth_stencil_info)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .push_next(&mut transparent_rendering_info);

        let transparent_pipeline = unsafe {
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

        Ok((pipeline_layout, graphics_pipeline, transparent_pipeline))
    }
}

// ============================================================================
// RUNTIME RENDER LOOP
// ============================================================================
impl VulkanContext {
    pub fn draw_frame(
        &mut self,
        extracted_view: Option<&ExtractedView>,
        extracted_meshes: Option<&ExtractedMeshes>,
    ) -> Result<(), Box<dyn Error>> {
        // 1. Prepare Data
        let geometry_setup = Self::prepare_geometry(extracted_view, extracted_meshes);

        // 2. Upload Buffer
        self.upload_geometry(&geometry_setup.vertices);

        // 3. Record & Submit Command Buffer
        self.record_and_submit_commands(
            extracted_view,
            geometry_setup.opaque_draws,
            geometry_setup.transparent_draws,
        )?;

        Ok(())
    }

    fn prepare_geometry(
        extracted_view: Option<&ExtractedView>,
        extracted_meshes: Option<&ExtractedMeshes>,
    ) -> FrameGeometry {
        let mut flattened_vertices = Vec::new();
        let mut opaque_draw_commands = Vec::new();
        let mut transparent_draw_commands = Vec::new();

        if let Some(meshes_res) = extracted_meshes {
            let mut current_offset = 0;
            for mesh in &meshes_res.meshes {
                // Pack Opaque
                if !mesh.opaque_vertices.is_empty() {
                    flattened_vertices.extend(&mesh.opaque_vertices);
                    let triangle_count = (mesh.opaque_vertices.len() / 3) as u32;
                    opaque_draw_commands.push((mesh.transform, current_offset, triangle_count));
                    current_offset += mesh.opaque_vertices.len() as u32;
                }

                // Pack Transparent (With Camera-Depth Sorting)
                if !mesh.transparent_vertices.is_empty() {
                    let cam_pos_world = extracted_view
                        .map(|v| v.camera_position)
                        .unwrap_or(glam::Vec3::ZERO);

                    let local_cam_pos = mesh.transform.inverse().transform_point3(cam_pos_world);

                    let mut triangles = Vec::with_capacity(mesh.transparent_vertices.len() / 3);
                    for i in (0..mesh.transparent_vertices.len()).step_by(3) {
                        let v0 = mesh.transparent_vertices[i];
                        let v1 = mesh.transparent_vertices[i + 1];
                        let v2 = mesh.transparent_vertices[i + 2];

                        let centroid = (v0.position.truncate()
                            + v1.position.truncate()
                            + v2.position.truncate())
                            / 3.0;
                        let dist_sq = centroid.distance_squared(local_cam_pos);

                        triangles.push(([v0, v1, v2], dist_sq));
                    }

                    triangles.sort_unstable_by(|a, b| {
                        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                    });

                    for (tri, _) in triangles {
                        flattened_vertices.extend_from_slice(&tri);
                    }

                    let triangle_count = (mesh.transparent_vertices.len() / 3) as u32;
                    transparent_draw_commands.push((
                        mesh.transform,
                        current_offset,
                        triangle_count,
                    ));
                    current_offset += mesh.transparent_vertices.len() as u32;
                }
            }
        }
        FrameGeometry {
            vertices: flattened_vertices,
            opaque_draws: opaque_draw_commands,
            transparent_draws: transparent_draw_commands,
        }
    }

    fn upload_geometry(&self, flattened_vertices: &[Vertex]) {
        if flattened_vertices.is_empty() {
            return;
        }

        if let Some(alloc) = &self.vertex_allocation
            && let Some(mapped_ptr) = self.vertex_allocation.as_ref().and_then(|a| a.mapped_ptr())
        {
            let upload_size = std::mem::size_of_val(flattened_vertices);

            assert!(
                upload_size <= alloc.size() as usize,
                "FATAL: Vertex geometry ({upload_size} bytes) exceeded Vulkan buffer size ({} bytes)!",
                alloc.size()
            );

            unsafe {
                std::ptr::copy_nonoverlapping(
                    flattened_vertices.as_ptr() as *const u8,
                    mapped_ptr.as_ptr() as *mut u8,
                    upload_size,
                );
            }
        }
    }

    fn record_and_submit_commands(
        &mut self,
        extracted_view: Option<&ExtractedView>,
        opaque_draw_commands: Vec<(Mat4, u32, u32)>,
        transparent_draw_commands: Vec<(Mat4, u32, u32)>,
    ) -> Result<(), Box<dyn Error>> {
        let device = &self.device;
        let swapchain_ext = &self.swapchain_ext;

        let view_proj = extracted_view
            .map(|v| v.view_projection)
            .unwrap_or(Mat4::IDENTITY);

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

            let depth_image_barrier = vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(
                    vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                        | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                )
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .image(self.depth_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    level_count: 1,
                    layer_count: 1,
                    ..Default::default()
                });

            device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_memory_barrier, depth_image_barrier],
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

            let depth_attachment_info = vk::RenderingAttachmentInfo::default()
                .image_view(self.depth_image_view)
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                });

            let rendering_info = vk::RenderingInfo::default()
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain_extent,
                })
                .layer_count(1)
                .color_attachments(std::slice::from_ref(&color_attachment_info))
                .depth_attachment(&depth_attachment_info);

            device.cmd_begin_rendering(self.command_buffer, &rendering_info);

            device.cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                std::slice::from_ref(&self.descriptor_set),
                &[],
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

            // Draw Opaque First
            device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline,
            );
            for (model_matrix, vertex_offset, triangle_count) in opaque_draw_commands {
                let push_constants = MeshPushConstants {
                    mvp: view_proj * model_matrix,
                    vertex_offset,
                    _padding: [0; 3],
                };
                let matrix_bytes = bytemuck::bytes_of(&push_constants);
                device.cmd_push_constants(
                    self.command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::MESH_EXT,
                    0,
                    matrix_bytes,
                );
                self.mesh_ext
                    .cmd_draw_mesh_tasks(self.command_buffer, triangle_count, 1, 1);
            }

            // Draw Transparent Last
            device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.transparent_pipeline,
            );
            for (model_matrix, vertex_offset, triangle_count) in transparent_draw_commands {
                let push_constants = MeshPushConstants {
                    mvp: view_proj * model_matrix,
                    vertex_offset,
                    _padding: [0; 3],
                };
                let matrix_bytes = bytemuck::bytes_of(&push_constants);
                device.cmd_push_constants(
                    self.command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::MESH_EXT,
                    0,
                    matrix_bytes,
                );
                self.mesh_ext
                    .cmd_draw_mesh_tasks(self.command_buffer, triangle_count, 1, 1);
            }

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

// ============================================================================
// CLEANUP
// ============================================================================
impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();

            if let Some(alloc) = self.vertex_allocation.take() {
                self.allocator.free(alloc).unwrap();
            }
            if let Some(alloc) = self.depth_allocation.take() {
                self.allocator.free(alloc).unwrap();
            }
            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.destroy_image(self.depth_image, None);

            self.device.destroy_buffer(self.vertex_buffer, None);

            std::mem::ManuallyDrop::drop(&mut self.allocator);

            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);

            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline(self.transparent_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);

            self.device.destroy_fence(self.in_flight_fence, None);
            self.device
                .destroy_semaphore(self.render_finished_semaphore, None);
            self.device
                .destroy_semaphore(self.image_available_semaphore, None);

            self.device.destroy_command_pool(self.command_pool, None);

            for &view in &self.swapchain_image_views {
                self.device.destroy_image_view(view, None);
            }

            self.swapchain_ext.destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_ext.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);

            tracing::info!("Vulkan Context destroyed cleanly.");
        }
    }
}
