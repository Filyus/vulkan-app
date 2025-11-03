//! Error handling module for the Vulkan App application.
//! 
//! This module defines custom error types for different components of the application,
//! providing better error context and making debugging easier.

use std::fmt;

/// Custom error type for the entire application
#[derive(Debug)]
pub enum AppError {
    /// Vulkan-related errors
    Vulkan(VulkanError),
    
    /// Window-related errors
    Window(WindowError),
    
    /// ECS-related errors
    ECS(EcsError),
    
    /// IO-related errors
    IO(std::io::Error),
    
    /// Generic errors with custom messages
    Generic(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Vulkan(err) => write!(f, "Vulkan error: {}", err),
            AppError::Window(err) => write!(f, "Window error: {}", err),
            AppError::ECS(err) => write!(f, "ECS error: {}", err),
            AppError::IO(err) => write!(f, "IO error: {}", err),
            AppError::Generic(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// Vulkan-specific errors
#[derive(Debug)]
#[allow(dead_code)] // Some error variants are for future error handling
pub enum VulkanError {
    /// Instance creation failed
    InstanceCreation(String),
    
    /// Device creation failed
    DeviceCreation(String),
    
    /// Surface creation failed
    SurfaceCreation(String),
    
    /// Swapchain creation failed
    SwapchainCreation(String),
    
    /// Pipeline creation failed
    PipelineCreation(String),
    
    /// Buffer creation failed
    BufferCreation(String),
    
    /// Memory allocation failed
    MemoryAllocation(String),
    
    /// Shader compilation failed
    ShaderCompilation(String),
    
    /// Command buffer recording failed
    CommandBuffer(String),
    
    /// Rendering failed
    Rendering(String),
    
    /// Validation layer error
    Validation(String),
}

impl fmt::Display for VulkanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VulkanError::InstanceCreation(msg) => write!(f, "Instance creation failed: {}", msg),
            VulkanError::DeviceCreation(msg) => write!(f, "Device creation failed: {}", msg),
            VulkanError::SurfaceCreation(msg) => write!(f, "Surface creation failed: {}", msg),
            VulkanError::SwapchainCreation(msg) => write!(f, "Swapchain creation failed: {}", msg),
            VulkanError::PipelineCreation(msg) => write!(f, "Pipeline creation failed: {}", msg),
            VulkanError::BufferCreation(msg) => write!(f, "Buffer creation failed: {}", msg),
            VulkanError::MemoryAllocation(msg) => write!(f, "Memory allocation failed: {}", msg),
            VulkanError::ShaderCompilation(msg) => write!(f, "Shader compilation failed: {}", msg),
            VulkanError::CommandBuffer(msg) => write!(f, "Command buffer error: {}", msg),
            VulkanError::Rendering(msg) => write!(f, "Rendering error: {}", msg),
            VulkanError::Validation(msg) => write!(f, "Validation layer error: {}", msg),
        }
    }
}

impl std::error::Error for VulkanError {}

/// Window-related errors
#[derive(Debug)]
#[allow(dead_code)] // Some error variants are for future error handling
pub enum WindowError {
    /// Window creation failed
    Creation(String),
    
    /// Event loop error
    EventLoop(String),
    
    /// Surface handle error
    SurfaceHandle(String),
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowError::Creation(msg) => write!(f, "Window creation failed: {}", msg),
            WindowError::EventLoop(msg) => write!(f, "Event loop error: {}", msg),
            WindowError::SurfaceHandle(msg) => write!(f, "Surface handle error: {}", msg),
        }
    }
}

impl std::error::Error for WindowError {}

/// ECS-related errors
#[derive(Debug)]
#[allow(dead_code)] // Some error variants are for future error handling
pub enum EcsError {
    /// World initialization failed
    WorldInitialization(String),
    
    /// System execution failed
    SystemExecution(String),
    
    /// Resource access failed
    ResourceAccess(String),
    
    /// Entity creation failed
    EntityCreation(String),
}

impl fmt::Display for EcsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EcsError::WorldInitialization(msg) => write!(f, "World initialization failed: {}", msg),
            EcsError::SystemExecution(msg) => write!(f, "System execution failed: {}", msg),
            EcsError::ResourceAccess(msg) => write!(f, "Resource access failed: {}", msg),
            EcsError::EntityCreation(msg) => write!(f, "Entity creation failed: {}", msg),
        }
    }
}

impl std::error::Error for EcsError {}

// Conversion from ash::vk::Result to our custom error type
impl From<ash::vk::Result> for AppError {
    fn from(result: ash::vk::Result) -> Self {
        match result {
            ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => {
                AppError::Vulkan(VulkanError::MemoryAllocation("Out of host memory".to_string()))
            }
            ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => {
                AppError::Vulkan(VulkanError::MemoryAllocation("Out of device memory".to_string()))
            }
            ash::vk::Result::ERROR_INITIALIZATION_FAILED => {
                AppError::Vulkan(VulkanError::InstanceCreation("Initialization failed".to_string()))
            }
            ash::vk::Result::ERROR_DEVICE_LOST => {
                AppError::Vulkan(VulkanError::DeviceCreation("Device lost".to_string()))
            }
            ash::vk::Result::ERROR_SURFACE_LOST_KHR => {
                AppError::Vulkan(VulkanError::SurfaceCreation("Surface lost".to_string()))
            }
            _ => AppError::Vulkan(VulkanError::Rendering(format!("Vulkan error: {:?}", result))),
        }
    }
}

// Conversion from std::io::Error to our custom error type
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IO(err)
    }
}

// Conversion from Box<dyn std::error::Error> to our custom error type
impl From<Box<dyn std::error::Error>> for AppError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        AppError::Generic(err.to_string())
    }
}

// Conversion from VulkanError to AppError
impl From<VulkanError> for AppError {
    fn from(err: VulkanError) -> Self {
        AppError::Vulkan(err)
    }
}

// Conversion from EcsError to AppError
impl From<EcsError> for AppError {
    fn from(err: EcsError) -> Self {
        AppError::ECS(err)
    }
}

// Conversion from WindowError to AppError
impl From<WindowError> for AppError {
    fn from(err: WindowError) -> Self {
        AppError::Window(err)
    }
}

// Conversion from winit::error::EventLoopError to AppError
impl From<winit::error::EventLoopError> for AppError {
    fn from(err: winit::error::EventLoopError) -> Self {
        AppError::Window(WindowError::EventLoop(err.to_string()))
    }
}

// Conversion from winit::error::OsError to AppError
impl From<winit::error::OsError> for AppError {
    fn from(err: winit::error::OsError) -> Self {
        AppError::Window(WindowError::Creation(err.to_string()))
    }
}

/// Result type alias for our application
pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_display() {
        let vulkan_err = VulkanError::InstanceCreation("Failed to create instance".to_string());
        let app_err = AppError::Vulkan(vulkan_err);
        
        let display_str = format!("{}", app_err);
        assert_eq!(display_str, "Vulkan error: Instance creation failed: Failed to create instance");
    }

    #[test]
    fn test_app_error_from_vk_result() {
        let result = ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY;
        let app_err: AppError = result.into();
        
        match app_err {
            AppError::Vulkan(VulkanError::MemoryAllocation(msg)) => {
                assert_eq!(msg, "Out of host memory");
            }
            _ => panic!("Expected MemoryAllocation error"),
        }
    }

    #[test]
    fn test_app_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let app_err: AppError = io_err.into();
        
        match app_err {
            AppError::IO(_) => {}, // Expected
            _ => panic!("Expected IO error"),
        }
    }

    #[test]
    fn test_app_error_from_box_error() {
        let boxed_err: Box<dyn std::error::Error> =
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Some error"));
        let app_err: AppError = boxed_err.into();
        
        match app_err {
            AppError::Generic(msg) => {
                assert!(msg.contains("Some error"));
            }
            _ => panic!("Expected Generic error"),
        }
    }

    #[test]
    fn test_app_error_from_vulkan_error() {
        let vulkan_err = VulkanError::DeviceCreation("Device creation failed".to_string());
        let app_err: AppError = vulkan_err.into();
        
        match app_err {
            AppError::Vulkan(VulkanError::DeviceCreation(msg)) => {
                assert_eq!(msg, "Device creation failed");
            }
            _ => panic!("Expected Vulkan error with DeviceCreation variant"),
        }
    }

    #[test]
    fn test_app_error_from_ecs_error() {
        let ecs_err = EcsError::EntityCreation("Failed to create entity".to_string());
        let app_err: AppError = ecs_err.into();
        
        match app_err {
            AppError::ECS(EcsError::EntityCreation(msg)) => {
                assert_eq!(msg, "Failed to create entity");
            }
            _ => panic!("Expected ECS error"),
        }
    }

    #[test]
    fn test_app_error_from_window_error() {
        let window_err = WindowError::Creation("Window creation failed".to_string());
        let app_err: AppError = window_err.into();
        
        match app_err {
            AppError::Window(WindowError::Creation(msg)) => {
                assert_eq!(msg, "Window creation failed");
            }
            _ => panic!("Expected Window error"),
        }
    }

    #[test]
    fn test_result_type_alias() {
        // Test that Result<T> works correctly
        let ok_result: Result<String> = Ok("success".to_string());
        assert!(ok_result.is_ok());
        
        let err_result: Result<String> = Err(AppError::Generic("error".to_string()));
        assert!(err_result.is_err());
    }
}