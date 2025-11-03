//! Debug and logging module for the Vulkan App application.
//! 
//! This module provides debugging utilities, logging infrastructure, and validation
//! helpers to make debugging and supporting the application easier.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use log::{debug, info, warn, error};
use crate::error::{Result, VulkanAppError, VulkanError};

/// Debug utilities for Vulkan objects
#[allow(dead_code)] // Fields and methods are for future debugging features
pub struct VulkanDebugUtils {
    /// Object names for debugging
    object_names: HashMap<u64, String>,
    
    /// Debug messenger callback
    #[cfg(debug_assertions)]
    debug_messenger: Option<ash::extensions::ext::DebugUtils>,
    
    /// Debug messenger handle
    #[cfg(debug_assertions)]
    messenger_handle: Option<ash::vk::DebugUtilsMessengerEXT>,
    
    /// Frame time tracking
    frame_times: Vec<Duration>,
    
    /// Last frame time
    last_frame_time: Option<Instant>,
}

#[allow(dead_code)]
impl VulkanDebugUtils {
    /// Create a new debug utilities instance
    pub fn new() -> Self {
        Self {
            object_names: HashMap::new(),
            #[cfg(debug_assertions)]
            debug_messenger: None,
            #[cfg(debug_assertions)]
            messenger_handle: None,
            frame_times: Vec::new(),
            last_frame_time: None,
        }
    }
    
    /// Set up debug messenger for validation layers
    #[cfg(debug_assertions)]
    pub fn setup_debug_messenger(
        &mut self, 
        entry: &ash::Entry, 
        instance: &ash::Instance
    ) -> Result<()> {
        use crate::config::vulkan;
        
        if !vulkan::ENABLE_VALIDATION_LAYERS {
            return Ok(());
        }
        
        let debug_utils = ash::extensions::ext::DebugUtils::new(entry, instance);
        
        let create_info = ash::vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR |
                ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
                ash::vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                ash::vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            )
            .message_type(
                ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
                ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION |
                ash::vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            )
            .pfn_user_callback(Some(vulkan_debug_callback));
            
        let messenger = unsafe {
            debug_utils.create_debug_utils_messenger(&create_info, None)
                .map_err(|e| VulkanAppError::Vulkan(
                    VulkanError::Validation(format!("Failed to create debug messenger: {:?}", e))
                ))?
        };
        
        self.debug_messenger = Some(debug_utils);
        self.messenger_handle = Some(messenger);
        info!("Debug messenger created successfully");
        Ok(())
    }
    
    /// Clean up debug messenger
    #[cfg(debug_assertions)]
    pub fn cleanup_debug_messenger(&mut self) {
        if let (Some(_debug_utils), Some(messenger)) = (self.debug_messenger.take(), self.messenger_handle.take()) {
            unsafe {
                _debug_utils.destroy_debug_utils_messenger(messenger, None);
            }
            info!("Debug messenger cleaned up");
        }
    }
    
    /// Set a debug name for a Vulkan object
    pub fn set_object_name<T>(&mut self, _device: &ash::Device, object: T, name: &str)
    where
        T: ash::vk::Handle + Copy
    {
        #[cfg(debug_assertions)]
        {
            
            // Note: This is a simplified implementation
            // In a real application, you would need to properly implement object naming
            let raw_handle = object.as_raw();
            self.object_names.insert(raw_handle, name.to_string());
            debug!("Set debug name '{}' for object {:?}", name, raw_handle);
        }
    }
    
    /// Begin a new frame for timing
    pub fn begin_frame(&mut self) {
        if crate::config::debug::ENABLE_FRAME_TIME_TRACKING {
            self.last_frame_time = Some(Instant::now());
        }
    }
    
    /// End the current frame and record the time
    pub fn end_frame(&mut self) {
        if crate::config::debug::ENABLE_FRAME_TIME_TRACKING {
            if let Some(last_time) = self.last_frame_time {
                let frame_time = last_time.elapsed();
                self.frame_times.push(frame_time);
                
                // Keep only the last 60 frames for averaging
                if self.frame_times.len() > 60 {
                    self.frame_times.remove(0);
                }
                
                // Log frame time every 60 frames
                if self.frame_times.len() == 60 {
                    let avg_time: Duration = self.frame_times.iter().sum();
                    let avg_time = avg_time / self.frame_times.len() as u32;
                    let fps = 1.0 / avg_time.as_secs_f32();
                    info!("Average frame time: {:?} ({} FPS)", avg_time, fps as u32);
                }
            }
        }
    }
    
    /// Get the average frame time over the recorded frames
    pub fn get_average_frame_time(&self) -> Option<Duration> {
        if self.frame_times.is_empty() {
            return None;
        }
        
        let total: Duration = self.frame_times.iter().sum();
        Some(total / self.frame_times.len() as u32)
    }
    
    /// Log memory usage information
    pub fn log_memory_usage(&self, _device: &ash::Device) {
        #[cfg(debug_assertions)]
        {
            if crate::config::debug::ENABLE_PERFORMANCE_MONITORING {
                // This is a simplified implementation
                // In a real application, you would query actual memory usage
                info!("Memory usage logging (simplified implementation)");
            }
        }
    }
    
    /// Validate that a Vulkan operation succeeded
    pub fn validate_vulkan_result(result: ash::vk::Result, operation: &str) -> Result<()> {
        if result != ash::vk::Result::SUCCESS {
            error!("Vulkan operation '{}' failed with result: {:?}", operation, result);
            return Err(VulkanAppError::Vulkan(
                VulkanError::Validation(format!("{} failed: {:?}", operation, result))
            ));
        }
        
        debug!("Vulkan operation '{}' succeeded", operation);
        Ok(())
    }
}

impl Default for VulkanDebugUtils {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for VulkanDebugUtils {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        self.cleanup_debug_messenger();
    }
}

/// Vulkan debug callback function
#[cfg(debug_assertions)]
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: ash::vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_types: ash::vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const ash::vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> ash::vk::Bool32 {
    use std::ffi::CStr;
    
    let message = CStr::from_ptr((*p_callback_data).p_message);
    let message = message.to_string_lossy();
    
    match message_severity {
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            error!("Vulkan Validation Error: {}", message);
        }
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            warn!("Vulkan Validation Warning: {}", message);
        }
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            info!("Vulkan Validation Info: {}", message);
        }
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            debug!("Vulkan Validation Verbose: {}", message);
        }
        _ => {
            debug!("Vulkan Validation: {}", message);
        }
    }
    
    ash::vk::FALSE
}

/// Initialize the logging system
pub fn init_logging() -> Result<()> {
    use crate::config::debug;
    
    if !debug::ENABLE_LOGGING {
        return Ok(());
    }
    
    // Simple console logger setup
    // In a real application, you might want to use a more sophisticated logging setup
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(debug::LOG_LEVEL)
        .chain(std::io::stdout())
        .apply()
        .map_err(|e| VulkanAppError::Generic(
            format!("Failed to initialize logging: {}", e)
        ))?;
    
    info!("Logging system initialized");
    Ok(())
}

/// Performance profiler for measuring execution time
#[allow(dead_code)] // Profiler utilities for future performance analysis
pub struct Profiler {
    /// Timed sections
    sections: HashMap<String, Vec<Duration>>,
    
    /// Currently running sections
    running_sections: HashMap<String, Instant>,
}

#[allow(dead_code)]
impl Profiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
            running_sections: HashMap::new(),
        }
    }
    
    /// Start timing a section
    pub fn start_section(&mut self, name: &str) {
        if crate::config::debug::ENABLE_PERFORMANCE_MONITORING {
            self.running_sections.insert(name.to_string(), Instant::now());
            debug!("Started profiling section: {}", name);
        }
    }
    
    /// End timing a section
    pub fn end_section(&mut self, name: &str) {
        if crate::config::debug::ENABLE_PERFORMANCE_MONITORING {
            if let Some(start_time) = self.running_sections.remove(name) {
                let duration = start_time.elapsed();
                self.sections.entry(name.to_string())
                    .or_insert_with(Vec::new)
                    .push(duration);
                
                debug!("Ended profiling section: {} (took {:?})", name, duration);
            }
        }
    }
    
    /// Get the average time for a section
    pub fn get_average_time(&self, name: &str) -> Option<Duration> {
        if let Some(times) = self.sections.get(name) {
            if times.is_empty() {
                return None;
            }
            
            let total: Duration = times.iter().sum();
            Some(total / times.len() as u32)
        } else {
            None
        }
    }
    
    /// Print a summary of all profiled sections
    pub fn print_summary(&self) {
        if crate::config::debug::ENABLE_PERFORMANCE_MONITORING {
            info!("Performance Profile Summary:");
            
            for (name, times) in &self.sections {
                if let Some(avg) = self.get_average_time(name) {
                    info!("  {}: {:?} ({} samples)", name, avg, times.len());
                }
            }
        }
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII helper for profiling a scope
#[allow(dead_code)] // Scoped profiler for future performance analysis
pub struct ScopedProfiler<'a> {
    profiler: &'a mut Profiler,
    name: String,
}

#[allow(dead_code)]
impl<'a> ScopedProfiler<'a> {
    /// Create a new scoped profiler
    pub fn new(profiler: &'a mut Profiler, name: &str) -> Self {
        profiler.start_section(name);
        Self {
            profiler,
            name: name.to_string(),
        }
    }
}

impl<'a> Drop for ScopedProfiler<'a> {
    fn drop(&mut self) {
        self.profiler.end_section(&self.name);
    }
}

/// Macro for easy profiling of a code block
#[macro_export]
macro_rules! profile_scope {
    ($profiler:expr, $name:expr, $block:block) => {
        let _profiler = $crate::debug::ScopedProfiler::new($profiler, $name);
        $block
    };
}