use crate::core::{device::VulkanDevice, swapchain::SwapchainSetup, sync::SyncSetup};
use crate::pipeline::{descriptor::DescriptorSetup, graphics::GraphicsPipelines};
use crate::resource::{allocator::GpuAllocator, buffer::GeometryArena, texture::DepthTexture};

use anantham_core::render_bridge::components::{ExtractedMeshes, ExtractedView, Vertex};
use ash::vk;
use bevy_ecs::prelude::Resource;
use glam::Mat4;
use std::error::Error;
use winit::window::Window;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshPushConstants {
    pub mvp: glam::Mat4,
    pub vertex_offset: u32,
    pub _padding: [u32; 3],
}

struct FrameGeometry {
    vertices: Vec<Vertex>,
    opaque_draws: Vec<(Mat4, u32, u32)>,
    transparent_draws: Vec<(Mat4, u32, u32)>,
}

#[derive(Resource)]
pub struct VulkanContext {
    pub vkd: VulkanDevice,
    pub swapchain: SwapchainSetup,
    pub sync: SyncSetup,

    pub allocator: GpuAllocator,
    pub geometry_arena: GeometryArena,
    pub depth_texture: DepthTexture,

    pub descriptors: DescriptorSetup,
    pub pipelines: GraphicsPipelines,
}

impl VulkanContext {
    pub fn new(window: &Window) -> Result<Self, Box<dyn Error>> {
        tracing::debug!("Starting Vulkan boot sequence...");

        let vkd = VulkanDevice::new(window)?;
        let swapchain = SwapchainSetup::new(window, &vkd)?;
        let sync = SyncSetup::new(&vkd)?;

        let mut allocator = GpuAllocator::new(
            vkd.instance.clone(),
            vkd.device.clone(),
            vkd.physical_device,
        )?;

        let vertex_buffer = GeometryArena::new(&vkd.device, &mut allocator.inner)?;

        let depth_format = vk::Format::D32_SFLOAT;
        let depth_texture = DepthTexture::new(
            &vkd.device,
            &mut allocator.inner,
            swapchain.extent,
            depth_format,
        )?;

        let descriptors = DescriptorSetup::new(&vkd.device, vertex_buffer.buffer)?;
        let pipelines = GraphicsPipelines::new(
            &vkd.device,
            descriptors.layout,
            swapchain.format,
            depth_format,
        )?;

        tracing::info!("Vulkan Context fully initialized");

        Ok(Self {
            vkd,
            swapchain,
            sync,
            allocator,
            geometry_arena: vertex_buffer,
            depth_texture,
            descriptors,
            pipelines,
        })
    }

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

        if let Some(alloc) = &self.geometry_arena.allocation
            && let Some(mapped_ptr) = alloc.mapped_ptr()
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
        let device = &self.vkd.device;
        let swapchain_ext = &self.swapchain.ext;

        let view_proj = extracted_view
            .map(|v| v.view_projection)
            .unwrap_or(Mat4::IDENTITY);

        unsafe {
            device.wait_for_fences(&[self.sync.in_flight], true, u64::MAX)?;
            device.reset_fences(&[self.sync.in_flight])?;

            let (image_index, _) = swapchain_ext.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                self.sync.image_available,
                vk::Fence::null(),
            )?;

            device.reset_command_buffer(
                self.sync.command_buffer,
                vk::CommandBufferResetFlags::empty(),
            )?;

            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device.begin_command_buffer(self.sync.command_buffer, &begin_info)?;

            let image = self.swapchain.images[image_index as usize];
            let image_view = self.swapchain.image_views[image_index as usize];

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
                .image(self.depth_texture.image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    level_count: 1,
                    layer_count: 1,
                    ..Default::default()
                });

            device.cmd_pipeline_barrier(
                self.sync.command_buffer,
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
                .image_view(self.depth_texture.view)
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
                    extent: self.swapchain.extent,
                })
                .layer_count(1)
                .color_attachments(std::slice::from_ref(&color_attachment_info))
                .depth_attachment(&depth_attachment_info);

            device.cmd_begin_rendering(self.sync.command_buffer, &rendering_info);

            device.cmd_bind_descriptor_sets(
                self.sync.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipelines.layout,
                0,
                std::slice::from_ref(&self.descriptors.set),
                &[],
            );

            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.swapchain.extent.width as f32,
                height: self.swapchain.extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            device.cmd_set_viewport(self.sync.command_buffer, 0, &[viewport]);

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            };
            device.cmd_set_scissor(self.sync.command_buffer, 0, &[scissor]);

            // Draw Opaque First
            device.cmd_bind_pipeline(
                self.sync.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipelines.opaque,
            );
            for (model_matrix, vertex_offset, triangle_count) in opaque_draw_commands {
                let push_constants = MeshPushConstants {
                    mvp: view_proj * model_matrix,
                    vertex_offset,
                    _padding: [0; 3],
                };
                let matrix_bytes = bytemuck::bytes_of(&push_constants);
                device.cmd_push_constants(
                    self.sync.command_buffer,
                    self.pipelines.layout,
                    vk::ShaderStageFlags::MESH_EXT,
                    0,
                    matrix_bytes,
                );
                self.vkd.mesh_ext.cmd_draw_mesh_tasks(
                    self.sync.command_buffer,
                    triangle_count,
                    1,
                    1,
                );
            }

            // Draw Transparent Last
            device.cmd_bind_pipeline(
                self.sync.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipelines.transparent,
            );
            for (model_matrix, vertex_offset, triangle_count) in transparent_draw_commands {
                let push_constants = MeshPushConstants {
                    mvp: view_proj * model_matrix,
                    vertex_offset,
                    _padding: [0; 3],
                };
                let matrix_bytes = bytemuck::bytes_of(&push_constants);
                device.cmd_push_constants(
                    self.sync.command_buffer,
                    self.pipelines.layout,
                    vk::ShaderStageFlags::MESH_EXT,
                    0,
                    matrix_bytes,
                );
                self.vkd.mesh_ext.cmd_draw_mesh_tasks(
                    self.sync.command_buffer,
                    triangle_count,
                    1,
                    1,
                );
            }

            device.cmd_end_rendering(self.sync.command_buffer);

            image_memory_barrier.src_access_mask = vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
            image_memory_barrier.dst_access_mask = vk::AccessFlags::empty();
            image_memory_barrier.old_layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
            image_memory_barrier.new_layout = vk::ImageLayout::PRESENT_SRC_KHR;

            device.cmd_pipeline_barrier(
                self.sync.command_buffer,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&image_memory_barrier),
            );

            device.end_command_buffer(self.sync.command_buffer)?;

            let wait_semaphores = [self.sync.image_available];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.sync.command_buffer];
            let signal_semaphores = [self.sync.render_finished];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            device.queue_submit(self.vkd.graphics_queue, &[submit_info], self.sync.in_flight)?;

            let swapchains = [self.swapchain.swapchain];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            swapchain_ext.queue_present(self.vkd.graphics_queue, &present_info)?;
        }
        Ok(())
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            let device = &self.vkd.device;
            let _ = device.device_wait_idle();

            self.geometry_arena
                .destroy(device, &mut self.allocator.inner);
            self.depth_texture
                .destroy(device, &mut self.allocator.inner);

            device.destroy_descriptor_pool(self.descriptors.pool, None);
            device.destroy_descriptor_set_layout(self.descriptors.layout, None);
            device.destroy_pipeline(self.pipelines.opaque, None);
            device.destroy_pipeline(self.pipelines.transparent, None);
            device.destroy_pipeline_layout(self.pipelines.layout, None);

            device.destroy_fence(self.sync.in_flight, None);
            device.destroy_semaphore(self.sync.render_finished, None);
            device.destroy_semaphore(self.sync.image_available, None);
            device.destroy_command_pool(self.sync.command_pool, None);

            for &view in &self.swapchain.image_views {
                device.destroy_image_view(view, None);
            }
            self.swapchain
                .ext
                .destroy_swapchain(self.swapchain.swapchain, None);

            device.destroy_device(None);
            self.vkd.surface_ext.destroy_surface(self.vkd.surface, None);
            self.vkd.instance.destroy_instance(None);

            tracing::info!("Vulkan Context destroyed cleanly.");
        }
    }
}
