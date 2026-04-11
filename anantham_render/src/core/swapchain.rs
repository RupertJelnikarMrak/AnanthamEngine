use super::device::VulkanDevice;
use ash::vk;
use std::error::Error;
use winit::window::Window;

pub struct SwapchainSetup {
    pub ext: ash::khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
}

impl SwapchainSetup {
    pub fn new(window: &Window, vkd: &VulkanDevice) -> Result<Self, Box<dyn Error>> {
        let capabilities = unsafe {
            vkd.surface_ext
                .get_physical_device_surface_capabilities(vkd.physical_device, vkd.surface)?
        };
        let formats = unsafe {
            vkd.surface_ext
                .get_physical_device_surface_formats(vkd.physical_device, vkd.surface)?
        };
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
            .surface(vkd.surface)
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

        let swapchain_ext = ash::khr::swapchain::Device::new(&vkd.instance, &vkd.device);
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
                unsafe { vkd.device.create_image_view(&view_info, None).unwrap() }
            })
            .collect();

        Ok(Self {
            ext: swapchain_ext,
            swapchain,
            images: swapchain_images,
            image_views: swapchain_image_views,
            format: format.format,
            extent,
        })
    }
}
