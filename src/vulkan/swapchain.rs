use ash::vk;
use ash::{Device, Instance, Entry};
use crate::vulkan::device::{VulkanDevice, QueueFamilyIndices};
use crate::error::{Result, VulkanError};
use winit::window::Window;
use log::{debug, info};

/// Vulkan swapchain wrapper with proper resource management
///
/// This struct manages the Vulkan swapchain, images, and image views,
/// ensuring proper cleanup and providing debugging capabilities.
pub struct VulkanSwapchain {
    /// The swapchain
    pub swapchain: vk::SwapchainKHR,
    
    /// The swapchain images (owned by the swapchain)
    pub _swapchain_images: Vec<vk::Image>,
    
    /// The swapchain image format
    pub swapchain_image_format: vk::Format,
    
    /// The swapchain extent
    pub swapchain_extent: vk::Extent2D,
    
    /// The swapchain image views
    pub swapchain_image_views: Vec<vk::ImageView>,
    
    /// The swapchain loader
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    
    /// The device reference for cleanup
    pub _device: Device,
}

impl VulkanSwapchain {
    /// Create a new Vulkan swapchain
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance
    /// * `entry` - The Vulkan entry point
    /// * `device` - The Vulkan device
    /// * `surface` - The surface to present to
    /// * `window` - The window
    ///
    /// # Returns
    /// A new VulkanSwapchain instance
    ///
    /// # Errors
    /// Returns an error if swapchain creation fails
    pub fn new(
        instance: &Instance,
        entry: &Entry,
        device: &VulkanDevice,
        surface: vk::SurfaceKHR,
        window: &Window,
    ) -> Result<Self> {
        info!("Creating Vulkan swapchain");
        
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
        
        debug!("Swapchain created with {} images", swapchain_images.len());
        
        let swapchain_image_views = Self::create_swapchain_image_views(
            &device.device,
            &swapchain_images,
            swapchain_image_format
        )?;
        
        debug!("Created {} image views", swapchain_image_views.len());
        
        info!("Vulkan swapchain created successfully");
        
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
    
    /// Create a swapchain
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance
    /// * `entry` - The Vulkan entry point
    /// * `_device` - The logical device
    /// * `physical_device` - The physical device
    /// * `surface` - The surface to present to
    /// * `swapchain_loader` - The swapchain loader
    /// * `window` - The window
    /// * `queue_families` - The queue family indices
    ///
    /// # Returns
    /// A tuple of (swapchain, swapchain_images, swapchain_image_format, swapchain_extent)
    ///
    /// # Errors
    /// Returns an error if swapchain creation fails
    fn create_swapchain(
        instance: &Instance,
        entry: &Entry,
        _device: &Device,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        swapchain_loader: &ash::extensions::khr::Swapchain,
        window: &Window,
        queue_families: &QueueFamilyIndices,
    ) -> Result<(vk::SwapchainKHR, Vec<vk::Image>, vk::Format, vk::Extent2D)> {
        debug!("Creating swapchain");
        
        let surface_loader_temp = ash::extensions::khr::Surface::new(entry, instance);
        let surface_capabilities = unsafe {
            surface_loader_temp.get_physical_device_surface_capabilities(physical_device, surface)
                .map_err(|e| VulkanError::SwapchainCreation(format!("Failed to get surface capabilities: {:?}", e)))?
        };
        
        let surface_formats = unsafe {
            surface_loader_temp.get_physical_device_surface_formats(physical_device, surface)
                .map_err(|e| VulkanError::SwapchainCreation(format!("Failed to get surface formats: {:?}", e)))?
        };
        
        let present_modes = unsafe {
            surface_loader_temp.get_physical_device_surface_present_modes(physical_device, surface)
                .map_err(|e| VulkanError::SwapchainCreation(format!("Failed to get present modes: {:?}", e)))?
        };
        
        debug!("Found {} surface formats and {} present modes", surface_formats.len(), present_modes.len());
        
        let surface_format = surface_formats.iter()
            .find(|format| format.format == vk::Format::B8G8R8A8_SRGB && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .unwrap_or(&surface_formats[0]);
        
        debug!("Selected surface format: {:?}", surface_format.format);
        
        let present_mode = present_modes.iter()
            .find(|mode| **mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(&vk::PresentModeKHR::FIFO);
        
        debug!("Selected present mode: {:?}", present_mode);
        
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
        
        debug!("Selected swapchain extent: {}x{}", extent.width, extent.height);
        
        let image_count = surface_capabilities.min_image_count + 1;
        let image_count = if surface_capabilities.max_image_count > 0 {
            image_count.min(surface_capabilities.max_image_count)
        } else {
            image_count
        };
        
        debug!("Selected image count: {}", image_count);
        
        let (sharing_mode, queue_family_indices) = if queue_families.graphics_family != queue_families.present_family {
            let indices_vec = vec![queue_families.graphics_family.unwrap(), queue_families.present_family.unwrap()];
            debug!("Using concurrent sharing mode for different queue families");
            (vk::SharingMode::CONCURRENT, indices_vec)
        } else {
            debug!("Using exclusive sharing mode for same queue family");
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
        
        let swapchain = unsafe {
            swapchain_loader.create_swapchain(&create_info, None)
                .map_err(|e| VulkanError::SwapchainCreation(format!("Failed to create swapchain: {:?}", e)))?
        };
        let swapchain_images = unsafe {
            swapchain_loader.get_swapchain_images(swapchain)
                .map_err(|e| VulkanError::SwapchainCreation(format!("Failed to get swapchain images: {:?}", e)))?
        };
        
        debug!("Swapchain created successfully");
        Ok((swapchain, swapchain_images, surface_format.format, extent))
    }
    
    /// Create image views for the swapchain images
    ///
    /// # Arguments
    /// * `device` - The Vulkan device
    /// * `images` - The swapchain images
    /// * `format` - The image format
    ///
    /// # Returns
    /// A vector of created image views
    ///
    /// # Errors
    /// Returns an error if image view creation fails
    fn create_swapchain_image_views(
        device: &Device,
        images: &[vk::Image],
        format: vk::Format
    ) -> Result<Vec<vk::ImageView>> {
        debug!("Creating {} image views", images.len());
        
        let mut image_views = vec![];
        
        for (i, &image) in images.iter().enumerate() {
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
            
            let image_view = unsafe {
                device.create_image_view(&create_info, None)
                    .map_err(|e| VulkanError::SwapchainCreation(format!("Failed to create image view {}: {:?}", i, e)))?
            };
            image_views.push(image_view);
        }
        
        debug!("Image views created successfully");
        Ok(image_views)
    }
}

impl Drop for VulkanSwapchain {
    fn drop(&mut self) {
        debug!("Destroying Vulkan swapchain");
        unsafe {
            // Destroy image views first
            for &image_view in &self.swapchain_image_views {
                self._device.destroy_image_view(image_view, None);
            }
            // Then destroy the swapchain
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
        }
        debug!("Vulkan swapchain destroyed");
    }
}