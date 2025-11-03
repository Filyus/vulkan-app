use legion::{Resources, Schedule, World};
use crate::ecs::systems::{create_sdf_entities, sdf_render_system, transform_update_system};
use crate::vulkan::renderer::VulkanRenderer;
use crate::error::{Result, AppError, EcsError};
use crate::hud::{HUD, HUDConfig, ToolbarPosition};
use log::{info, error, debug};
use winit::window::Window;
use ash::vk;

/// ECS World that manages entities, components, and systems
pub struct ECSWorld {
    /// The legion World that holds all entities and components
    pub world: World,
    
    /// Resources that can be accessed by systems
    pub resources: Resources,
    
    /// The schedule of systems to execute each frame
    pub schedule: Schedule,
    
    /// HUD system for toolbar and UI
    pub hud: Option<HUD>,
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
        info!("=== ECSWorld::new() STARTED ===");
        let mut world = World::default();
        let mut resources = Resources::default();
        
        info!("Inserting Vulkan renderer as resource");
        // Insert the Vulkan renderer as a resource
        resources.insert(vulkan_renderer);
        
        info!("Inserting SDF entity tracker vector");
        // Insert a vector to track SDF entities
        resources.insert(Vec::<legion::Entity>::new());
        
        info!("Creating SDF entities");
        // Create SDF entities once during initialization
        create_sdf_entities(&mut world, &mut resources)
            .map_err(|e| {
                error!("Failed to create SDF entities: {}", e);
                EcsError::EntityCreation(format!("Failed to create SDF entities: {}", e))
            })?;
        
        info!("Creating ECS schedule");
        // Create the schedule with systems that run every frame
        let schedule = Schedule::builder()
            .add_thread_local_fn(transform_update_system)
            .add_thread_local_fn(sdf_render_system)
            .build();
        
        info!("ECS world created successfully");
        info!("=== ECSWorld::new() COMPLETED ===");
        
        Ok(Self {
            world,
            resources,
            schedule,
            hud: None,
        })
    }
    
    /// Initialize HUD system with the given window
    ///
    /// # Arguments
    /// * `window` - The window to associate with the HUD
    ///
    /// # Returns
    /// * Ok(()) if HUD initialization succeeded
    /// * Err if HUD initialization failed
    pub fn init_hud(
        &mut self,
        window: &Window,
    ) -> Result<()> {
        info!("=== HUD INITIALIZATION STARTED ===");
        
        info!("Getting VulkanRenderer from resources");
        let vulkan_renderer = self.resources.get::<VulkanRenderer>()
            .ok_or_else(|| {
                error!("VulkanRenderer not found in ECS resources");
                EcsError::ResourceAccess("VulkanRenderer not found for HUD initialization".to_string())
            })?;
        
        info!("Creating HUD config");
        let config = HUDConfig::default();
        let render_pass = vulkan_renderer.pipeline.render_pass;
        let device = &vulkan_renderer.device;
        
        info!("Creating HUD instance with window, device, and render pass");
        let mut hud = HUD::new(
            window,
            device,
            &*vulkan_renderer,
            render_pass,
            config,
        ).map_err(|e| {
            error!("Failed to create HUD instance: {}", e);
            AppError::HUD(format!("Failed to initialize HUD: {}", e))
        })?;
        
        info!("HUD instance created, initializing font texture");
        hud.init_font_texture()
            .map_err(|e| {
                error!("Failed to initialize HUD font texture: {}", e);
                AppError::HUD(format!("Failed to initialize HUD font texture: {}", e))
            })?;
        
        info!("Font texture initialized, storing HUD in ECS world");
        // Store HUD in the world
        self.hud = Some(hud);
        
        info!("HUD system initialized successfully with font texture");
        debug!("HUD stored in ECS world at: {:p}", self.hud.as_ref().unwrap());
        info!("=== HUD INITIALIZATION COMPLETED ===");
        Ok(())
    }
    
    /// Execute all systems in the schedule
    ///
    /// # Arguments
    /// * `window` - Current window for HUD input handling
    /// * `delta_time` - Time since last frame
    ///
    /// # Returns
    /// * Ok(()) if all systems executed successfully
    /// * Err if any system failed to execute
    pub fn execute(&mut self, window: &Window, delta_time: f32) -> Result<()> {
        // Update HUD first
        if let Some(ref mut hud) = self.hud {
            hud.update(window, delta_time);
        }
        
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
        
        // Check if HUD is available and log its state
        match self.hud {
            Some(ref mut hud) => {
                info!("Drawing frame with HUD - HUD is available");
                debug!("HUD address: {:p}", hud);
                vulkan_renderer.draw_frame_with_hud(hud)
                    .map_err(|e| AppError::Vulkan(crate::error::VulkanError::Rendering(
                        format!("Failed to draw frame with HUD: {}", e)
                    )))?;
            }
            None => {
                info!("Drawing frame without HUD - HUD is None");
                vulkan_renderer.draw_frame()
                    .map_err(|e| AppError::Vulkan(crate::error::VulkanError::Rendering(
                        format!("Failed to draw frame: {}", e)
                    )))?;
            }
        }
        
        Ok(())
    }
    
    /// Handle window resize event
    ///
    /// # Arguments
    /// * `new_width` - The new window width
    /// * `new_height` - The new window height
    /// * `window` - The window for HUD resizing
    ///
    /// # Returns
    /// * Ok(()) if resize was handled successfully
    /// * Err if resize handling failed
    pub fn handle_window_resize(&mut self, new_width: u32, new_height: u32, _window: &Window) -> Result<()> {
        let mut vulkan_renderer = self.resources.get_mut::<VulkanRenderer>()
            .ok_or_else(|| EcsError::ResourceAccess("VulkanRenderer resource not found in ECS world".to_string()))?;
        
        vulkan_renderer.handle_resize(new_width, new_height)
            .map_err(|e| AppError::Vulkan(crate::error::VulkanError::Rendering(
                format!("Failed to handle window resize: {}", e)
            )))?;
        
        // Update HUD if available
        if let Some(ref mut hud) = self.hud {
            let extent = vk::Extent2D { width: new_width, height: new_height };
            hud.handle_resize(extent);
            
            // Re-initialize HUD if needed for major changes
            if new_width > 0 && new_height > 0 {
                // HUD will automatically handle resizing through platform interface
            }
        }
        
        Ok(())
    }
    
    /// Handle fullscreen toggle
    ///
    /// # Arguments
    /// * `window` - The window to handle fullscreen for
    ///
    /// # Returns
    /// * Ok(()) if fullscreen toggle was handled successfully
    /// * Err if fullscreen toggle handling failed
    pub fn handle_fullscreen_toggle(&mut self, window: &Window) -> Result<()> {
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
            )))?;
        
        // Update HUD for fullscreen
        if let Some(ref mut hud) = self.hud {
            let extent = vk::Extent2D { width: new_width, height: new_height };
            hud.handle_resize(extent);
        }
        
        Ok(())
    }
    
    /// Toggle HUD visibility
    pub fn toggle_hud(&mut self) {
        if let Some(ref mut hud) = self.hud {
            hud.toolbar.toggle_visibility();
            info!("HUD visibility toggled");
        }
    }
    
    /// Clean up HUD system manually
    /// This should be called before the Vulkan renderer is destroyed
    /// to ensure proper resource cleanup order
    pub fn cleanup_hud(&mut self) {
        if self.hud.is_some() {
            info!("Manually cleaning up HUD system");
            // Explicitly drop the HUD to trigger cleanup
            drop(std::mem::replace(&mut self.hud, None));
            info!("HUD system cleaned up manually");
        }
    }
    
    /// Set HUD toolbar position
    #[allow(dead_code)]
    pub fn set_hud_position(&mut self, position: ToolbarPosition) {
        if let Some(ref mut hud) = self.hud {
            hud.toolbar.set_position(crate::hud::toolbar::ToolbarPosition::Top); // Convert position type
            info!("HUD position set to {:?}", position);
        }
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