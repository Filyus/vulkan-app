use ash::vk;
use ash::{Device, Instance, Entry};
use crate::vulkan::device::{VulkanDevice, QueueFamilyIndices};
use winit::window::Window;

pub struct VulkanSwapchain {
    pub swapchain: vk::SwapchainKHR,
    pub _swapchain_images: Vec<vk::Image>,
    pub swapchain_image_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub _device: Device,
}

impl VulkanSwapchain {
    pub fn new(
        instance: &Instance,
        entry: &Entry,
        device: &VulkanDevice,
        surface: vk::SurfaceKHR,
        window: &Window,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, &device.device);
        
        let (swapchain, swapchain_images, swapchain_image_format, swapchain_extent) =
            Self::create_swapchain(
                instance,
                entry,
                &device.device,
                device.physical_device,
                surface,
                &swapchain_loader,
                window,
                &device.queue_families
            )?;
        
        let swapchain_image_views = Self::create_swapchain_image_views(
            &device.device, 
            &swapchain_images, 
            swapchain_image_format
        )?;
        
        Ok(Self {
            swapchain,
            _swapchain_images: swapchain_images,
            swapchain_image_format,
            swapchain_extent,
            swapchain_image_views,
            swapchain_loader,
            _device: device.device.clone(),
        })
    }
    
    fn create_swapchain(
        instance: &Instance,
        entry: &Entry,
        _device: &Device,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        swapchain_loader: &ash::extensions::khr::Swapchain,
        window: &Window,
        queue_families: &QueueFamilyIndices,
    ) -> Result<(vk::SwapchainKHR, Vec<vk::Image>, vk::Format, vk::Extent2D), Box<dyn std::error::Error>> {
        let surface_loader_temp = ash::extensions::khr::Surface::new(entry, instance);
        let surface_capabilities = unsafe {
            surface_loader_temp.get_physical_device_surface_capabilities(physical_device, surface)?
        };
        
        let surface_formats = unsafe {
            surface_loader_temp.get_physical_device_surface_formats(physical_device, surface)?
        };
        
        let present_modes = unsafe {
            surface_loader_temp.get_physical_device_surface_present_modes(physical_device, surface)?
        };
        
        let surface_format = surface_formats.iter()
            .find(|format| format.format == vk::Format::B8G8R8A8_SRGB && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .unwrap_or(&surface_formats[0]);
        
        let present_mode = present_modes.iter()
            .find(|mode| **mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(&vk::PresentModeKHR::FIFO);
        
        let extent = if surface_capabilities.current_extent.width != u32::MAX {
            surface_capabilities.current_extent
        } else {
            let size = window.inner_size();
            vk::Extent2D {
                width: size.width.clamp(
                    surface_capabilities.min_image_extent.width,
                    surface_capabilities.max_image_extent.width
                ),
                height: size.height.clamp(
                    surface_capabilities.min_image_extent.height,
                    surface_capabilities.max_image_extent.height
                ),
            }
        };
        
        let image_count = surface_capabilities.min_image_count + 1;
        let image_count = if surface_capabilities.max_image_count > 0 {
            image_count.min(surface_capabilities.max_image_count)
        } else {
            image_count
        };
        
        let (sharing_mode, queue_family_indices) = if queue_families.graphics_family != queue_families.present_family {
            let indices_vec = vec![queue_families.graphics_family.unwrap(), queue_families.present_family.unwrap()];
            (vk::SharingMode::CONCURRENT, indices_vec)
        } else {
            (vk::SharingMode::EXCLUSIVE, vec![])
        };
        
        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(*present_mode)
            .clipped(true)
            .image_sharing_mode(sharing_mode)
            .queue_family_indices(&queue_family_indices);
        
        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None)? };
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        
        Ok((swapchain, swapchain_images, surface_format.format, extent))
    }
    
    fn create_swapchain_image_views(
        device: &Device, 
        images: &[vk::Image], 
        format: vk::Format
    ) -> Result<Vec<vk::ImageView>, Box<dyn std::error::Error>> {
        let mut image_views = vec![];
        
        for &image in images {
            let create_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            
            let image_view = unsafe { device.create_image_view(&create_info, None)? };
            image_views.push(image_view);
        }
        
        Ok(image_views)
    }
}

impl Drop for VulkanSwapchain {
    fn drop(&mut self) {
        unsafe {
            // Destroy image views first
            for &image_view in &self.swapchain_image_views {
                self._device.destroy_image_view(image_view, None);
            }
            // Then destroy the swapchain
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
        }
    }
}