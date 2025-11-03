use legion::{Resources, Schedule, World};
use crate::ecs::systems::{create_sdf_entities, sdf_render_system, transform_update_system};
use crate::vulkan::renderer::VulkanRenderer;
use crate::error::{Result, AppError, EcsError};
use log::info;

/// ECS World that manages entities, components, and systems
pub struct ECSWorld {
    /// The legion World that holds all entities and components
    pub world: World,
    
    /// Resources that can be accessed by systems
    pub resources: Resources,
    
    /// The schedule of systems to execute each frame
    pub schedule: Schedule,
}

impl ECSWorld {
    /// Create a new ECS world with the given Vulkan renderer
    ///
    /// # Arguments
    /// * `vulkan_renderer` - The Vulkan renderer to use for rendering
    ///
    /// # Returns
    /// * A new ECS world instance
    ///
    /// # Errors
    /// Returns an error if world initialization fails
    pub fn new(vulkan_renderer: VulkanRenderer) -> Result<Self> {
        let mut world = World::default();
        let mut resources = Resources::default();
        
        // Insert the Vulkan renderer as a resource
        resources.insert(vulkan_renderer);
        
        // Insert a vector to track SDF entities
        resources.insert(Vec::<legion::Entity>::new());
        
        // Create SDF entities once during initialization
        create_sdf_entities(&mut world, &mut resources)
            .map_err(|e| EcsError::EntityCreation(format!("Failed to create SDF entities: {}", e)))?;
        
        // Create the schedule with systems that run every frame
        let schedule = Schedule::builder()
            .add_thread_local_fn(transform_update_system)
            .add_thread_local_fn(sdf_render_system)
            .build();
        
        info!("ECS world created successfully");
        
        Ok(Self {
            world,
            resources,
            schedule,
        })
    }
    
    /// Execute all systems in the schedule
    ///
    /// # Returns
    /// * Ok(()) if all systems executed successfully
    /// * Err if any system failed to execute
    pub fn execute(&mut self) -> Result<()> {
        self.schedule.execute(&mut self.world, &mut self.resources);
        Ok(())
    }
    
    /// Draw a single frame
    ///
    /// # Returns
    /// * Ok(()) if the frame was drawn successfully
    /// * Err if drawing failed
    pub fn draw_frame(&mut self) -> Result<()> {
        let mut vulkan_renderer = self.resources.get_mut::<VulkanRenderer>()
            .ok_or_else(|| EcsError::ResourceAccess("VulkanRenderer resource not found in ECS world".to_string()))?;
        
        vulkan_renderer.draw_frame()
            .map_err(|e| AppError::Vulkan(crate::error::VulkanError::Rendering(
                format!("Failed to draw frame: {}", e)
            )))
    }
    
    /// Handle window resize event
    ///
    /// # Arguments
    /// * `new_width` - The new window width
    /// * `new_height` - The new window height
    ///
    /// # Returns
    /// * Ok(()) if resize was handled successfully
    /// * Err if resize handling failed
    pub fn handle_window_resize(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        let mut vulkan_renderer = self.resources.get_mut::<VulkanRenderer>()
            .ok_or_else(|| EcsError::ResourceAccess("VulkanRenderer resource not found in ECS world".to_string()))?;
        
        vulkan_renderer.handle_resize(new_width, new_height)
            .map_err(|e| AppError::Vulkan(crate::error::VulkanError::Rendering(
                format!("Failed to handle window resize: {}", e)
            )))
    }
    
    /// Handle fullscreen toggle
    ///
    /// # Arguments
    /// * `window` - The window to handle fullscreen for
    ///
    /// # Returns
    /// * Ok(()) if fullscreen toggle was handled successfully
    /// * Err if fullscreen toggle handling failed
    pub fn handle_fullscreen_toggle(&mut self, window: &winit::window::Window) -> Result<()> {
        let mut vulkan_renderer = self.resources.get_mut::<VulkanRenderer>()
            .ok_or_else(|| EcsError::ResourceAccess("VulkanRenderer resource not found in ECS world".to_string()))?;
        
        // Get current window size
        let physical_size = window.inner_size();
        let new_width = physical_size.width;
        let new_height = physical_size.height;
        
        info!("Handling fullscreen toggle, new size: {}x{}", new_width, new_height);
        
        // Handle the resize which will recreate the swapchain
        vulkan_renderer.handle_resize(new_width, new_height)
            .map_err(|e| AppError::Vulkan(crate::error::VulkanError::Rendering(
                format!("Failed to handle fullscreen toggle: {}", e)
            )))
    }
    
    /// Get the number of entities in the world
    ///
    /// # Returns
    /// The number of entities currently in the world
    #[allow(dead_code)] // For future entity management
    pub fn entity_count(&self) -> usize {
        // Legion doesn't provide a direct way to count entities
        // This is a simplified implementation
        self.len()
    }
    
    /// Get a reference to the world
    ///
    /// # Returns
    /// A reference to the legion World
    #[allow(dead_code)] // For future world access
    pub fn world(&self) -> &World {
        &self.world
    }
    
    /// Get a mutable reference to the world
    ///
    /// # Returns
    /// A mutable reference to the legion World
    #[allow(dead_code)] // For future world access
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
    
    /// Get a reference to the resources
    ///
    /// # Returns
    /// A reference to the resources
    #[allow(dead_code)] // For future resource access
    pub fn resources(&self) -> &Resources {
        &self.resources
    }
    
    /// Get a mutable reference to the resources
    ///
    /// # Returns
    /// A mutable reference to the resources
    #[allow(dead_code)] // For future resource access
    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }
}

// Implement the legion World methods for convenience
impl std::ops::Deref for ECSWorld {
    type Target = World;
    
    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl std::ops::DerefMut for ECSWorld {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world
    }
}