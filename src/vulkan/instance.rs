use ash::vk;
use ash::{Entry, Instance};
use std::ffi::CString;
use crate::error::{Result, VulkanError};
use crate::config;
use log::{debug, info};
#[cfg(debug_assertions)]
use log::warn;

/// Vulkan instance wrapper with proper resource management
///
/// This struct manages the Vulkan instance and entry point, ensuring proper
/// cleanup and providing debugging capabilities.
pub struct VulkanInstance {
    /// The Ash entry point for Vulkan
    pub entry: Entry,
    
    /// The Vulkan instance
    pub instance: Instance,
    
    /// Debug utilities for validation and logging
    #[cfg(debug_assertions)]
    debug_utils: Option<crate::debug::VulkanDebugUtils>,
}

impl VulkanInstance {
    /// Create a new Vulkan instance
    ///
    /// # Returns
    /// A new VulkanInstance with proper initialization
    ///
    /// # Errors
    /// Returns an error if instance creation fails
    pub fn new() -> Result<Self> {
        info!("Creating Vulkan instance");
        
        let entry = unsafe { Entry::load() }
            .map_err(|e| VulkanError::InstanceCreation(format!("Failed to load Vulkan entry: {:?}", e)))?;
        
        debug!("Vulkan entry loaded successfully");
        
        let instance = Self::create_instance(&entry)?;
        debug!("Vulkan instance created successfully");
        
        #[cfg(debug_assertions)]
        let mut debug_utils = crate::debug::VulkanDebugUtils::new();
        #[cfg(debug_assertions)]
        if config::vulkan::ENABLE_VALIDATION_LAYERS {
            debug_utils.setup_debug_messenger(&entry, &instance)?;
        }
        
        info!("Vulkan instance initialized successfully");
        
        Ok(Self {
            entry,
            instance,
            #[cfg(debug_assertions)]
            debug_utils: Some(debug_utils),
        })
    }
    
    /// Create the Vulkan instance with proper configuration
    ///
    /// # Arguments
    /// * `entry` - The Vulkan entry point
    ///
    /// # Returns
    /// The created Vulkan instance
    ///
    /// # Errors
    /// Returns an error if instance creation fails
    fn create_instance(entry: &Entry) -> Result<Instance> {
        let app_name = CString::new(config::vulkan::APP_NAME)
            .map_err(|e| VulkanError::InstanceCreation(format!("Failed to create app name string: {}", e)))?;
        let engine_name = CString::new(config::vulkan::ENGINE_NAME)
            .map_err(|e| VulkanError::InstanceCreation(format!("Failed to create engine name string: {}", e)))?;

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(config::vulkan::APP_VERSION)
            .engine_name(&engine_name)
            .engine_version(config::vulkan::ENGINE_VERSION)
            .api_version(config::vulkan::API_VERSION);

        // Get required extensions
        let (extensions, _extension_strings) = Self::get_required_extensions(entry)?;
        
        // Check for validation layer support in debug builds
        #[cfg(debug_assertions)]
        let (layers, _layer_strings) = if config::vulkan::ENABLE_VALIDATION_LAYERS {
            Self::get_validation_layers(entry)?
        } else {
            (Vec::new(), Vec::new())
        };
        
        #[cfg(not(debug_assertions))]
        let (layers, _layer_strings): (Vec<*const i8>, Vec<CString>) = (Vec::new(), Vec::new());

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers);

        let instance = unsafe {
            entry.create_instance(&create_info, None)
                .map_err(|e| VulkanError::InstanceCreation(format!("Failed to create Vulkan instance: {:?}", e)))?
        };
        
        Ok(instance)
    }
    
    /// Get the list of required extensions for the instance
    ///
    /// # Arguments
    /// * `entry` - The Vulkan entry point
    ///
    /// # Returns
    /// A vector of required extension names
    ///
    /// # Errors
    /// Returns an error if extension enumeration fails
    #[allow(dead_code)]
    #[allow(unused_variables)]
    fn get_required_extensions(_entry: &Entry) -> Result<(Vec<*const i8>, Vec<CString>)> {
        let mut extensions = Vec::new();
        let mut extension_strings = Vec::new();
        
        // Get window extensions from winit
        // We'll use a placeholder for now since we don't have a window handle at instance creation time
        // In a real application, you would get the display handle from the window
        let display_handle = raw_window_handle::RawDisplayHandle::Windows(
            raw_window_handle::WindowsDisplayHandle::new()
        );
        let window_extensions = ash_window::enumerate_required_extensions(display_handle)
            .map_err(|e| VulkanError::InstanceCreation(format!("Failed to enumerate window extensions: {:?}", e)))?;
        
        // Convert window extensions to CStrings and store them
        for &ext in window_extensions {
            let ext_str = unsafe { std::ffi::CStr::from_ptr(ext) }.to_string_lossy().to_string();
            let ext_cstring = CString::new(ext_str)
                .map_err(|e| VulkanError::InstanceCreation(format!("Failed to create extension string: {}", e)))?;
            extensions.push(ext_cstring.as_ptr());
            extension_strings.push(ext_cstring);
        }
        
        debug!("Window extensions: {:?}", window_extensions);
        
        // Add debug utils extension in debug builds
        #[cfg(debug_assertions)]
        if config::vulkan::ENABLE_VALIDATION_LAYERS {
            if unsafe { _entry.enumerate_instance_extension_properties(None) }
                .map_err(|e| VulkanError::InstanceCreation(format!("Failed to enumerate instance extensions: {:?}", e)))?
                .iter()
                .any(|ext| {
                    let name = unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) };
                    name.to_str().unwrap() == ash::vk::EXT_DEBUG_UTILS_NAME.to_str().unwrap()
                }) {
                let debug_utils_name = CString::new(ash::vk::EXT_DEBUG_UTILS_NAME.to_str().unwrap())
                    .map_err(|e| VulkanError::InstanceCreation(format!("Failed to create debug utils string: {}", e)))?;
                extensions.push(debug_utils_name.as_ptr());
                extension_strings.push(debug_utils_name);
                debug!("Added debug utils extension");
            } else {
                warn!("Debug utils extension not available");
            }
        }
        
        // Convert to strings for debugging
        let extension_names: Vec<String> = extensions.iter()
            .map(|&ptr| unsafe { std::ffi::CStr::from_ptr(ptr) }.to_string_lossy().to_string())
            .collect();
        debug!("Required extensions: {:?}", extension_names);
        
        Ok((extensions, extension_strings))
    }
    
    /// Get the list of validation layers to enable
    ///
    /// # Arguments
    /// * `entry` - The Vulkan entry point
    ///
    /// # Returns
    /// A vector of validation layer names
    ///
    /// # Errors
    /// Returns an error if layer enumeration fails
    #[cfg(debug_assertions)]
    fn get_validation_layers(entry: &Entry) -> Result<(Vec<*const i8>, Vec<CString>)> {
        let available_layers = unsafe { entry.enumerate_instance_layer_properties() }
            .map_err(|e| VulkanError::InstanceCreation(format!("Failed to enumerate instance layers: {:?}", e)))?;
        
        debug!("Available validation layers:");
        for layer in &available_layers {
            let layer_name = unsafe { std::ffi::CStr::from_ptr(layer.layer_name.as_ptr()) };
            debug!("  {}", layer_name.to_string_lossy());
        }
        
        let mut layers = Vec::new();
        
        // Convert validation layer names to CStrings for proper null-termination
        let validation_layers_cstr: Vec<CString> = config::vulkan::VALIDATION_LAYERS
            .iter()
            .map(|&layer| CString::new(layer))
            .collect::<std::result::Result<Vec<CString>, _>>()
            .map_err(|e| VulkanError::InstanceCreation(format!("Failed to create layer string: {}", e)))?;
        
        for (i, layer_name) in config::vulkan::VALIDATION_LAYERS.iter().enumerate() {
            if available_layers.iter().any(|layer| {
                let name = unsafe { std::ffi::CStr::from_ptr(layer.layer_name.as_ptr()) };
                name.to_string_lossy() == *layer_name
            }) {
                layers.push(validation_layers_cstr[i].as_ptr());
                debug!("Enabling validation layer: {}", layer_name);
            } else {
                warn!("Validation layer '{}' not available", layer_name);
            }
        }
        
        if layers.is_empty() && config::vulkan::ENABLE_VALIDATION_LAYERS {
            warn!("No validation layers available, running without validation");
        }
        
        Ok((layers, validation_layers_cstr))
    }
    
    /// Get the debug utilities reference
    ///
    /// # Returns
    /// A reference to the debug utilities, if available
    #[cfg(debug_assertions)]
    #[allow(dead_code)] // For future debugging access
    pub fn debug_utils(&self) -> Option<&crate::debug::VulkanDebugUtils> {
        self.debug_utils.as_ref()
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        info!("Destroying Vulkan instance");
        
        #[cfg(debug_assertions)]
        if let Some(mut debug_utils) = self.debug_utils.take() {
            debug_utils.cleanup_debug_messenger();
        }
        
        unsafe {
            self.instance.destroy_instance(None);
        }
        
        debug!("Vulkan instance destroyed");
    }
}