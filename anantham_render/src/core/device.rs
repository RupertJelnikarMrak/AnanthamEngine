use ash::{Entry, Instance, ext::mesh_shader, vk};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::{error::Error, ffi::CStr};
use winit::window::Window;

pub struct VulkanDevice {
    pub entry: Entry,
    pub instance: Instance,
    pub surface_ext: ash::khr::surface::Instance,
    pub surface: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub graphics_queue: vk::Queue,
    pub graphics_queue_family_index: u32,
    pub mesh_ext: mesh_shader::Device,
}

impl VulkanDevice {
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
        let (physical_device, queue_family_index) = pdevices
            .into_iter()
            .filter_map(|pdevice| {
                let properties = unsafe { instance.get_physical_device_properties(pdevice) };
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

                queue_index.map(|index| {
                    let score = match properties.device_type {
                        vk::PhysicalDeviceType::DISCRETE_GPU => 1000,
                        vk::PhysicalDeviceType::INTEGRATED_GPU => 100,
                        _ => 0,
                    };
                    (score, pdevice, index as u32)
                })
            })
            .max_by_key(|&(score, _, _)| score)
            .map(|(_, pdevice, index)| (pdevice, index))
            .expect("Failed to find a suitable physical device with Mesh Shader support");

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
        let mesh_ext = mesh_shader::Device::new(&instance, &device);

        Ok(Self {
            entry,
            instance,
            surface_ext,
            surface,
            physical_device,
            device,
            graphics_queue,
            graphics_queue_family_index: queue_family_index,
            mesh_ext,
        })
    }
}
