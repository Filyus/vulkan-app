//! Configuration module for the Vulkan App application.
//! 
//! This module contains all configuration constants and settings for the application,
//! making it easier to modify behavior without changing code in multiple places.

/// Window configuration
pub mod window {
    /// Default window width in pixels
    pub const DEFAULT_WIDTH: u32 = 800;
    
    /// Default window height in pixels
    pub const DEFAULT_HEIGHT: u32 = 600;
    
    /// Window title
    pub const TITLE: &str = "Vulkan App - ECS";
    
    /// Minimum window width
    pub const MIN_WIDTH: u32 = 400;
    
    /// Minimum window height
    pub const MIN_HEIGHT: u32 = 300;
}

/// Windowed fullscreen configuration
pub mod windowed_fullscreen {
    /// Enable windowed fullscreen mode instead of true fullscreen
    pub const ENABLED: bool = true;
    
    /// Offset from screen edges to avoid overlapping with taskbar/dock
    pub const SCREEN_EDGE_OFFSET: u32 = 0;
}

/// Vulkan configuration
pub mod vulkan {
    /// Maximum number of frames that can be in flight
    pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
    
    /// Application name for Vulkan
    pub const APP_NAME: &str = "Vulkan App";
    
    /// Engine name for Vulkan
    pub const ENGINE_NAME: &str = "No Engine";
    
    /// Application version
    pub const APP_VERSION: u32 = ash::vk::make_api_version(0, 1, 0, 0);
    
    /// Engine version
    pub const ENGINE_VERSION: u32 = ash::vk::make_api_version(0, 1, 0, 0);
    
    /// API version
    pub const API_VERSION: u32 = ash::vk::make_api_version(0, 1, 0, 0);
    
    /// Enable validation layers in debug builds
    #[cfg(debug_assertions)]
    pub const ENABLE_VALIDATION_LAYERS: bool = true;
    
    /// Disable validation layers in release builds
    #[cfg(not(debug_assertions))]
    #[allow(dead_code)]
    pub const ENABLE_VALIDATION_LAYERS: bool = false;
    
    /// Validation layers to enable
    #[allow(dead_code)]
    pub const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_KHRONOS_validation"];
    
    /// Device extensions required
    pub const DEVICE_EXTENSIONS: &[&str] = &["VK_KHR_swapchain"];
    
}

/// Rendering configuration
pub mod rendering {
    /// Clear color for the framebuffer (R, G, B, A)
    pub const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    
    /// Default line width for rasterization
    pub const LINE_WIDTH: f32 = 1.0;
    
    
    /// Enable face culling
    pub const ENABLE_FACE_CULLING: bool = true;
    
    /// Cull mode for rasterization
    pub const CULL_MODE: ash::vk::CullModeFlags = ash::vk::CullModeFlags::BACK;
    
    /// Front face winding order
    pub const FRONT_FACE: ash::vk::FrontFace = ash::vk::FrontFace::CLOCKWISE;
}

/// Debug configuration
pub mod debug {
    /// Enable debug logging
    pub const ENABLE_LOGGING: bool = true;
    
    /// Minimum log level to display
    pub const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Debug;
    
    /// Enable verbose Vulkan debug messages
    #[allow(dead_code)] // For future verbose debugging
    pub const ENABLE_VERBOSE_VULKAN_DEBUG: bool = true;
    
    /// Enable performance monitoring
    #[allow(dead_code)] // For future performance analysis
    pub const ENABLE_PERFORMANCE_MONITORING: bool = false;
    
    /// Enable frame time tracking
    #[allow(dead_code)]
    pub const ENABLE_FRAME_TIME_TRACKING: bool = true;
    
    /// Enable debug mode with additional validation
    pub const ENABLE_DEBUG_MODE: bool = cfg!(debug_assertions);
    
    /// Enable detailed Vulkan validation
    pub const ENABLE_DETAILED_VALIDATION: bool = cfg!(debug_assertions);
    
    /// Enable object naming for debugging
    pub const ENABLE_OBJECT_NAMING: bool = cfg!(debug_assertions);
    
    /// Enable command buffer debugging
    pub const ENABLE_COMMAND_BUFFER_DEBUG: bool = cfg!(debug_assertions);
    
    /// Enable memory tracking
    pub const ENABLE_MEMORY_TRACKING: bool = cfg!(debug_assertions);
    
    /// Enable shader debugging
    pub const ENABLE_SHADER_DEBUGGING: bool = cfg!(debug_assertions);
    
    /// Enable pipeline debugging
    pub const ENABLE_PIPELINE_DEBUGGING: bool = cfg!(debug_assertions);
    
    /// Enable render pass debugging
    pub const ENABLE_RENDER_PASS_DEBUGGING: bool = cfg!(debug_assertions);
}

/// ECS configuration
pub mod ecs {
    /// Maximum number of entities that can be created
    #[allow(dead_code)] // For future entity management
    pub const MAX_ENTITIES: usize = 1000;
    
    /// Enable entity tracking for debugging
    #[allow(dead_code)] // For future entity debugging
    pub const ENABLE_ENTITY_TRACKING: bool = false;
    
    /// Enable system performance profiling
    #[allow(dead_code)] // For future system profiling
    pub const ENABLE_SYSTEM_PROFILING: bool = false;
}

/// Shader configuration
pub mod shader {
    /// Shader entry point name
    pub const ENTRY_POINT: &[u8] = b"main\0";
}

/// Memory configuration
pub mod memory {
    /// Buffer alignment requirements
    #[allow(dead_code)] // For future memory management
    pub const BUFFER_ALIGNMENT: u64 = 256;
    
    /// Memory allocation chunk size
    #[allow(dead_code)] // For future memory management
    pub const ALLOCATION_CHUNK_SIZE: u64 = 64 * 1024 * 1024; // 64MB
    
    /// Enable memory mapping debugging
    #[allow(dead_code)] // For future memory debugging
    pub const ENABLE_MEMORY_DEBUGGING: bool = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_config_constants() {
        assert_eq!(window::TITLE, "Vulkan App - ECS");
        assert_eq!(window::DEFAULT_WIDTH, 800);
        assert_eq!(window::DEFAULT_HEIGHT, 600);
        assert_eq!(window::MIN_WIDTH, 400);
        assert_eq!(window::MIN_HEIGHT, 300);
    }

    #[test]
    fn test_vulkan_config_constants() {
        assert_eq!(vulkan::APP_NAME, "Vulkan App");
        assert_eq!(vulkan::ENGINE_NAME, "No Engine");
        assert_eq!(vulkan::MAX_FRAMES_IN_FLIGHT, 2);
        assert!(!vulkan::VALIDATION_LAYERS.is_empty());
        assert!(!vulkan::DEVICE_EXTENSIONS.is_empty());
    }

    #[test]
    fn test_rendering_config_constants() {
        assert_eq!(rendering::CLEAR_COLOR, [0.0, 0.0, 0.0, 1.0]);
        assert!(rendering::ENABLE_FACE_CULLING);
        assert_eq!(rendering::LINE_WIDTH, 1.0);
        assert_eq!(rendering::CULL_MODE, ash::vk::CullModeFlags::BACK);
        assert_eq!(rendering::FRONT_FACE, ash::vk::FrontFace::CLOCKWISE);
    }

    #[test]
    fn test_debug_config_constants() {
        assert!(debug::ENABLE_LOGGING);
        assert!(debug::ENABLE_VERBOSE_VULKAN_DEBUG);
        assert!(!debug::ENABLE_PERFORMANCE_MONITORING);
        assert!(debug::ENABLE_FRAME_TIME_TRACKING);
        assert_eq!(debug::LOG_LEVEL, log::LevelFilter::Debug);
    }

    #[test]
    fn test_ecs_config_constants() {
        assert_eq!(ecs::MAX_ENTITIES, 1000);
        assert!(!ecs::ENABLE_ENTITY_TRACKING);
        assert!(!ecs::ENABLE_SYSTEM_PROFILING);
    }

    #[test]
    fn test_shader_config_constants() {
        assert_eq!(shader::ENTRY_POINT, b"main\0");
    }

    #[test]
    fn test_memory_config_constants() {
        assert_eq!(memory::BUFFER_ALIGNMENT, 256);
        assert_eq!(memory::ALLOCATION_CHUNK_SIZE, 64 * 1024 * 1024);
        assert!(!memory::ENABLE_MEMORY_DEBUGGING);
    }

    #[test]
    fn test_vulkan_api_version() {
        let expected_version = ash::vk::make_api_version(0, 1, 0, 0);
        assert_eq!(vulkan::API_VERSION, expected_version);
        assert_eq!(vulkan::APP_VERSION, expected_version);
        assert_eq!(vulkan::ENGINE_VERSION, expected_version);
    }

    #[test]
    fn test_validation_layers_config() {
        assert_eq!(vulkan::VALIDATION_LAYERS, &["VK_LAYER_KHRONOS_validation"]);
        assert_eq!(vulkan::DEVICE_EXTENSIONS, &["VK_KHR_swapchain"]);
    }
}