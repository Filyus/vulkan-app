use ash::vk;
use ash::{Device, Instance, Entry};
use std::ffi::{CStr, CString};
use crate::error::{Result, VulkanError};
use crate::config;
use log::{debug, info};

/// Queue family indices for graphics and presentation
#[derive(Clone, Debug, Default)]
pub struct QueueFamilyIndices {
    /// Graphics queue family index
    pub graphics_family: Option<u32>,
    
    /// Presentation queue family index
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    /// Check if all required queue families are found
    ///
    /// # Returns
    /// true if both graphics and present families are found
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

/// Vulkan device wrapper with proper resource management
///
/// This struct manages the Vulkan logical device, physical device, and queues,
/// ensuring proper cleanup and providing debugging capabilities.
pub struct VulkanDevice {
    /// The logical Vulkan device
    pub device: Device,
    
    /// The physical Vulkan device
    pub physical_device: vk::PhysicalDevice,
    
    /// The graphics queue
    pub graphics_queue: vk::Queue,
    
    /// The presentation queue
    pub present_queue: vk::Queue,
    
    /// Queue family indices
    pub queue_families: QueueFamilyIndices,
}

impl VulkanDevice {
    /// Create a new Vulkan device
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance
    /// * `entry` - The Vulkan entry point
    /// * `surface` - The surface to present to
    ///
    /// # Returns
    /// A new VulkanDevice instance
    ///
    /// # Errors
    /// Returns an error if device creation fails
    pub fn new(instance: &Instance, entry: &Entry, surface: vk::SurfaceKHR) -> Result<Self> {
        info!("Creating Vulkan device");
        
        let surface_loader = ash::khr::surface::Instance::new(entry, instance);
        
        let (physical_device, queue_families) = Self::pick_physical_device(instance, entry, &surface_loader, surface)?;
        
        let (device, graphics_queue, present_queue) = Self::create_logical_device(
            instance,
            physical_device,
            &queue_families
        )?;
        
        info!("Vulkan device created successfully");
        
        Ok(Self {
            device,
            physical_device,
            graphics_queue,
            present_queue,
            queue_families,
        })
    }
    
    /// Pick a suitable physical device
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance
    /// * `entry` - The Vulkan entry point
    /// * `surface_loader` - The surface loader
    /// * `surface` - The surface to present to
    ///
    /// # Returns
    /// A tuple of (physical_device, queue_families)
    ///
    /// # Errors
    /// Returns an error if no suitable device is found
    fn pick_physical_device(
        instance: &Instance,
        _entry: &Entry,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR
    ) -> Result<(vk::PhysicalDevice, QueueFamilyIndices)> {
        debug!("Enumerating physical devices");
        
        let devices = unsafe {
            instance.enumerate_physical_devices()
                .map_err(|e| VulkanError::DeviceCreation(format!("Failed to enumerate physical devices: {:?}", e)))?
        };
        
        debug!("Found {} physical devices", devices.len());
        
        for (i, &device) in devices.iter().enumerate() {
            let indices = Self::find_queue_families(instance, device, surface_loader, surface);
            if indices.is_complete() {
                let properties = unsafe { instance.get_physical_device_properties(device) };
                let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) };
                info!("Selected physical device {}: {}", i, device_name.to_string_lossy());
                return Ok((device, indices));
            }
        }
        
        Err(VulkanError::DeviceCreation("No suitable physical device found".to_string()).into())
    }
    
    /// Find queue families for a physical device
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance
    /// * `device` - The physical device
    /// * `surface_loader` - The surface loader
    /// * `surface` - The surface to present to
    ///
    /// # Returns
    /// Queue family indices
    fn find_queue_families(
        instance: &Instance,
        device: vk::PhysicalDevice,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR
    ) -> QueueFamilyIndices {
        debug!("Finding queue families for physical device");
        
        let mut indices = QueueFamilyIndices::default();
        
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(device) };
        
        debug!("Found {} queue families", queue_families.len());
        
        for (i, queue_family) in queue_families.iter().enumerate() {
            debug!("Queue family {}: flags={:?}", i, queue_family.queue_flags);
            
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i as u32);
                debug!("Found graphics queue family: {}", i);
            }
            
            let present_support = unsafe {
                surface_loader.get_physical_device_surface_support(device, i as u32, surface)
                    .unwrap_or(false)
            };
            
            if present_support {
                indices.present_family = Some(i as u32);
                debug!("Found present queue family: {}", i);
            }
            
            if indices.is_complete() {
                debug!("All required queue families found");
                break;
            }
        }
        
        indices
    }
    
    /// Create a logical device from a physical device
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance
    /// * `physical_device` - The physical device
    /// * `indices` - The queue family indices
    ///
    /// # Returns
    /// A tuple of (device, graphics_queue, present_queue)
    ///
    /// # Errors
    /// Returns an error if device creation fails
    fn create_logical_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        indices: &QueueFamilyIndices
    ) -> Result<(Device, vk::Queue, vk::Queue)> {
        debug!("Creating logical device");
        
        let queue_priorities = [1.0];
        
        let mut queue_create_infos = vec![];
        
        let queue_create_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(indices.graphics_family.unwrap())
            .queue_priorities(&queue_priorities);
        queue_create_infos.push(queue_create_info);
        
        if indices.graphics_family != indices.present_family {
            let queue_create_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(indices.present_family.unwrap())
                .queue_priorities(&queue_priorities);
            queue_create_infos.push(queue_create_info);
            debug!("Using separate queues for graphics and presentation");
        } else {
            debug!("Using same queue for graphics and presentation");
        }
        
        // Convert extension names to CStrings for proper null-termination
        let device_extensions_cstr: Vec<CString> = config::vulkan::DEVICE_EXTENSIONS
            .iter()
            .map(|&ext| CString::new(ext))
            .collect::<std::result::Result<Vec<CString>, _>>()
            .map_err(|e| VulkanError::DeviceCreation(format!("Failed to create extension string: {}", e)))?;
        
        // Convert to raw pointers
        let device_extensions: Vec<*const i8> = device_extensions_cstr
            .iter()
            .map(|ext| ext.as_ptr())
            .collect();
        
        debug!("Device extensions: {:?}", config::vulkan::DEVICE_EXTENSIONS);
        
        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions);
        
        let device = unsafe {
            instance.create_device(physical_device, &create_info, None)
                .map_err(|e| VulkanError::DeviceCreation(format!("Failed to create logical device: {:?}", e)))?
        };
        
        let graphics_queue = unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };
        let present_queue = unsafe { device.get_device_queue(indices.present_family.unwrap(), 0) };
        
        debug!("Logical device created successfully");
        Ok((device, graphics_queue, present_queue))
    }
    
    /// Get the name of the physical device
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance
    ///
    /// # Returns
    /// The device name as a string
    pub fn get_device_name(&self, instance: &Instance) -> String {
        let properties = unsafe {
            instance.get_physical_device_properties(self.physical_device)
        };
        
        let device_name = unsafe {
            CStr::from_ptr(properties.device_name.as_ptr())
        };
        
        device_name.to_string_lossy().to_string()
    }
    
    /// Get the properties of the physical device
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance
    ///
    /// # Returns
    /// The physical device properties
    #[allow(dead_code)] // For future device queries
    pub fn get_device_properties(&self, instance: &Instance) -> vk::PhysicalDeviceProperties {
        unsafe {
            instance.get_physical_device_properties(self.physical_device)
        }
    }
    
    /// Get the memory properties of the physical device
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance
    ///
    /// # Returns
    /// The physical device memory properties
    #[allow(dead_code)] // For future memory management
    pub fn get_memory_properties(&self, instance: &Instance) -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            instance.get_physical_device_memory_properties(self.physical_device)
        }
    }
    
    /// Check if the device supports a given extension
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance
    /// * `extension_name` - The name of the extension to check
    ///
    /// # Returns
    /// true if the extension is supported, false otherwise
    #[allow(dead_code)] // For future extension checking
    pub fn supports_extension(&self, instance: &Instance, extension_name: &str) -> bool {
        let extensions = unsafe {
            instance.enumerate_device_extension_properties(self.physical_device)
                .unwrap_or_default()
        };
        
        extensions.iter().any(|ext| {
            let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
            name.to_string_lossy() == extension_name
        })
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        debug!("Destroying Vulkan device");
        unsafe {
            self.device.destroy_device(None);
        }
        debug!("Vulkan device destroyed");
    }
}