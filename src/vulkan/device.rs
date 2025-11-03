use ash::vk;
use ash::{Device, Instance, Entry};
use std::ffi::CStr;

#[derive(Clone, Debug, Default)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub struct VulkanDevice {
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub queue_families: QueueFamilyIndices,
}

impl VulkanDevice {
    pub fn new(instance: &Instance, entry: &Entry, surface: vk::SurfaceKHR) -> Result<Self, Box<dyn std::error::Error>> {
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
        
        let (physical_device, queue_families) = Self::pick_physical_device(instance, entry, &surface_loader, surface)?;
        
        let (device, graphics_queue, present_queue) = Self::create_logical_device(
            instance, 
            physical_device, 
            &queue_families
        )?;
        
        Ok(Self {
            device,
            physical_device,
            graphics_queue,
            present_queue,
            queue_families,
        })
    }
    
    fn pick_physical_device(
        instance: &Instance,
        _entry: &Entry,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR
    ) -> Result<(vk::PhysicalDevice, QueueFamilyIndices), Box<dyn std::error::Error>> {
        let devices = unsafe { instance.enumerate_physical_devices()? };
        
        for &device in &devices {
            let indices = Self::find_queue_families(instance, device, surface_loader, surface);
            if indices.is_complete() {
                return Ok((device, indices));
            }
        }
        
        Err("No suitable physical device found".into())
    }
    
    fn find_queue_families(
        instance: &Instance, 
        device: vk::PhysicalDevice, 
        surface_loader: &ash::extensions::khr::Surface, 
        surface: vk::SurfaceKHR
    ) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices::default();
        
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(device) };
        
        for (i, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i as u32);
            }
            
            let present_support = unsafe {
                surface_loader.get_physical_device_surface_support(device, i as u32, surface)
            }.unwrap_or(false);
            
            if present_support {
                indices.present_family = Some(i as u32);
            }
            
            if indices.is_complete() {
                break;
            }
        }
        
        indices
    }
    
    fn create_logical_device(
        instance: &Instance, 
        physical_device: vk::PhysicalDevice, 
        indices: &QueueFamilyIndices
    ) -> Result<(Device, vk::Queue, vk::Queue), Box<dyn std::error::Error>> {
        let queue_priorities = [1.0];
        
        let mut queue_create_infos = vec![];
        
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(indices.graphics_family.unwrap())
            .queue_priorities(&queue_priorities);
        queue_create_infos.push(queue_create_info.build());
        
        if indices.graphics_family != indices.present_family {
            let queue_create_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(indices.present_family.unwrap())
                .queue_priorities(&queue_priorities);
            queue_create_infos.push(queue_create_info.build());
        }
        
        let device_extensions = vec![ash::extensions::khr::Swapchain::name().as_ptr()];
        
        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions);
        
        let device = unsafe { instance.create_device(physical_device, &create_info, None)? };
        
        let graphics_queue = unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };
        let present_queue = unsafe { device.get_device_queue(indices.present_family.unwrap(), 0) };
        
        Ok((device, graphics_queue, present_queue))
    }
    
    pub fn get_device_name(&self, instance: &Instance) -> String {
        let properties = unsafe {
            instance.get_physical_device_properties(self.physical_device)
        };
        
        let device_name = unsafe {
            CStr::from_ptr(properties.device_name.as_ptr())
        };
        
        device_name.to_string_lossy().to_string()
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}