use ash::vk;
use ash::{Entry, Instance};
use std::ffi::CString;

pub struct VulkanInstance {
    pub entry: Entry,
    pub instance: Instance,
}

impl VulkanInstance {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let entry = unsafe { Entry::load()? };
        let instance = Self::create_instance(&entry)?;
        
        Ok(Self { entry, instance })
    }
    
    fn create_instance(entry: &Entry) -> Result<Instance, Box<dyn std::error::Error>> {
        let app_name = CString::new("Vulkan Triangle Demo")?;
        let engine_name = CString::new("No Engine")?;

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let extensions = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::Win32Surface::name().as_ptr(),
        ];

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions);

        let instance = unsafe { entry.create_instance(&create_info, None)? };
        Ok(instance)
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}