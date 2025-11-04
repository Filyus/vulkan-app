use legion::{Resources, Schedule, World};
use std::sync::{Arc, Mutex};
use crate::ecs::systems::{create_sdf_entities, sdf_render_system, transform_update_system};
use crate::vulkan::renderer::VulkanRenderer;
use crate::vulkan::shader_compiler::ShaderCompiler;
use crate::vulkan::shader_watcher::{HotReloadManager, HotReloadConfig};
use crate::error::{Result, AppError, EcsError};
use crate::hud::{HUD, HUDConfig, ToolbarPosition};
use log::{info, error, debug, warn};
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
    
    /// Hot reload manager for shader changes
    pub hot_reload_manager: Option<HotReloadManager>,
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
        // Insert the Vulkan renderer as an Arc<Mutex> resource for shared mutable access
        let vulkan_renderer_arc = Arc::new(Mutex::new(vulkan_renderer));
        resources.insert(vulkan_renderer_arc);
        
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
            hot_reload_manager: None,
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
        let vulkan_renderer = self.resources.get::<Arc<Mutex<VulkanRenderer>>>()
            .ok_or_else(|| {
                error!("VulkanRenderer not found in ECS resources");
                EcsError::ResourceAccess("VulkanRenderer not found for HUD initialization".to_string())
            })?;

        info!("Creating HUD config");
        let config = HUDConfig::default();

        // Lock the renderer to access its data
        let renderer_guard = vulkan_renderer.lock().unwrap();
        let pipeline_guard = renderer_guard.pipeline.lock().unwrap();
        let render_pass = pipeline_guard.render_pass;
        let device = &renderer_guard.device;
        drop(pipeline_guard); // Release pipeline lock

        info!("Creating HUD instance with window, device, and render pass");
        let mut hud = HUD::new(
            window,
            device,
            &*renderer_guard,
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
    
    /// Initialize hot reload manager
    ///
    /// # Returns
    /// * Ok(()) if hot reload initialization succeeded
    /// * Err if hot reload initialization failed
    pub fn init_hot_reload(&mut self) -> Result<()> {
        info!("=== HOT RELOAD INITIALIZATION STARTED ===");

        // Get Vulkan renderer to access pipeline
        let vulkan_renderer = self.resources.get::<Arc<Mutex<VulkanRenderer>>>()
            .ok_or_else(|| {
                error!("VulkanRenderer not found in ECS resources");
                EcsError::ResourceAccess("VulkanRenderer not found for hot reload initialization".to_string())
            })?;

        // Create shader compiler
        let shader_compiler = Arc::new(Mutex::new(ShaderCompiler::new()?));

        // Create hot reload config
        let config = HotReloadConfig::default();

        // Create hot reload manager
        let mut hot_reload_manager = HotReloadManager::new(config, Arc::clone(&shader_compiler));

        // Initialize hot reload manager with pipeline - SHARE the same Arc!
        let renderer_guard = vulkan_renderer.lock().unwrap();
        let pipeline_arc = Arc::clone(&renderer_guard.pipeline);
        drop(renderer_guard); // Release lock before setting callback

        hot_reload_manager.initialize(pipeline_arc)?;

        // Store hot reload manager
        self.hot_reload_manager = Some(hot_reload_manager);

        info!("Hot reload manager initialized successfully with pipeline integration and immediate command buffer updates");
        info!("=== HOT RELOAD INITIALIZATION COMPLETED ===");
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
        // Get hot reload state before borrowing HUD
        let hot_reload_enabled = self.is_hot_reload_enabled();

        // Update HUD first
        if let Some(ref mut hud) = self.hud {
            hud.update(window, delta_time);
            // Update hot reload button state to match current hot reload status
            hud.toolbar.update_hot_reload_button_state(hot_reload_enabled);
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
        // Check if we need to update command buffers due to hot reload from previous frame
        // This MUST be done at the very beginning of the frame, before any rendering
        let _needs_command_buffer_update = if let Some(ref hot_reload_manager) = self.hot_reload_manager {
            let should_update = hot_reload_manager.check_and_clear_reloads_occurred();
            debug!("Command buffer update needed: {}", should_update);
            should_update
        } else {
            false
        };

        // Process any pending shader reloads first and check if pipeline was recreated
        let pipeline_was_recreated = if let Some(ref mut hot_reload_manager) = self.hot_reload_manager {
            match hot_reload_manager.process_pending_reloads() {
                Ok(was_recreated) => was_recreated,
                Err(e) => {
                    error!("Failed to process pending shader reloads: {}", e);
                    false // Continue with frame rendering even if reload fails
                }
            }
        } else {
            false
        };

        let vulkan_renderer = self.resources.get::<Arc<Mutex<VulkanRenderer>>>()
            .ok_or_else(|| EcsError::ResourceAccess("VulkanRenderer resource not found in ECS world".to_string()))?;

        // IMMEDIATE command buffer update if pipeline was recreated
        if pipeline_was_recreated {
            info!("Pipeline was recreated during hot reload, updating command buffers immediately");
            let mut renderer_guard = vulkan_renderer.lock().unwrap();
            if let Err(e) = renderer_guard.update_command_buffers_after_hot_reload() {
                error!("Failed to update command buffers after hot reload: {}", e);
                // Continue with rendering even if command buffer update fails
            } else {
                info!("Command buffer update completed successfully after hot reload");
            }
        }

        // Check if HUD is available and log its state
        let mut renderer_guard = vulkan_renderer.lock().unwrap();
        match self.hud {
            Some(ref mut hud) => {
                debug!("Drawing frame with HUD");
                debug!("HUD address: {:p}", hud);
                renderer_guard.draw_frame_with_hud(hud)
                    .map_err(|e| AppError::Vulkan(crate::error::VulkanError::Rendering(
                        format!("Failed to draw frame with HUD: {}", e)
                    )))?;
            }
            None => {
                debug!("Drawing frame without HUD");
                renderer_guard.draw_frame()
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
        let vulkan_renderer = self.resources.get::<Arc<Mutex<VulkanRenderer>>>()
            .ok_or_else(|| EcsError::ResourceAccess("VulkanRenderer resource not found in ECS world".to_string()))?;

        let mut renderer_guard = vulkan_renderer.lock().unwrap();
        renderer_guard.handle_resize(new_width, new_height)
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
        let vulkan_renderer = self.resources.get::<Arc<Mutex<VulkanRenderer>>>()
            .ok_or_else(|| EcsError::ResourceAccess("VulkanRenderer resource not found in ECS world".to_string()))?;

        // Get current window size
        let physical_size = window.inner_size();
        let new_width = physical_size.width;
        let new_height = physical_size.height;

        info!("Handling fullscreen toggle, new size: {}x{}", new_width, new_height);

        // Handle the resize which will recreate the swapchain
        let mut renderer_guard = vulkan_renderer.lock().unwrap();
        renderer_guard.handle_resize(new_width, new_height)
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
    
    /// Wait for GPU to complete all pending operations
    /// This should be called before resource cleanup to ensure no command buffers are in use
    pub fn wait_for_gpu_idle(&mut self) -> Result<()> {
        info!("Waiting for GPU to complete all pending operations");
        
        let vulkan_renderer = self.resources.get::<Arc<Mutex<VulkanRenderer>>>()
            .ok_or_else(|| EcsError::ResourceAccess("VulkanRenderer resource not found in ECS world".to_string()))?;
        
        let renderer_guard = vulkan_renderer.lock().unwrap();
        unsafe {
            renderer_guard.device.device.device_wait_idle()
                .map_err(|e| {
                    error!("Failed to wait for GPU idle: {:?}", e);
                    EcsError::ResourceAccess(format!("Failed to wait for GPU idle: {:?}", e))
                })?;
        }
        
        info!("GPU idle confirmed, safe to proceed with resource cleanup");
        Ok(())
    }
    
    /// Clean up HUD system manually
    /// This should be called before the Vulkan renderer is destroyed
    /// to ensure proper resource cleanup order
    pub fn cleanup_hud(&mut self) {
        if self.hud.is_some() {
            info!("Manually cleaning up HUD system");
            // Explicitly drop the HUD to trigger cleanup
            // The HUD's Drop implementation will handle proper resource cleanup
            drop(std::mem::replace(&mut self.hud, None));
            info!("HUD system cleaned up manually");
        }
    }

    /// Clean up hot reload manager manually
    /// This should be called before the Vulkan renderer is destroyed
    /// to break reference cycles and ensure proper pipeline cleanup
    pub fn cleanup_hot_reload(&mut self) {
        if self.hot_reload_manager.is_some() {
            info!("Manually cleaning up hot reload manager");
            // Explicitly drop the hot reload manager to break reference cycles
            drop(std::mem::replace(&mut self.hot_reload_manager, None));
            info!("Hot reload manager cleaned up manually");
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
    
    /// Enable or disable hot reload
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable hot reload
    ///
    /// # Returns
    /// * Ok(()) if operation succeeded
    /// * Err if operation failed
    pub fn set_hot_reload_enabled(&mut self, enabled: bool) -> Result<()> {
        if let Some(ref mut hot_reload) = self.hot_reload_manager {
            hot_reload.set_enabled(enabled)?;
            info!("Hot reload {}", if enabled { "enabled" } else { "disabled" });
        } else {
            warn!("Hot reload manager not initialized");
        }
        Ok(())
    }
    
    /// Check if hot reload is enabled
    ///
    /// # Returns
    /// true if hot reload is enabled, false otherwise
    pub fn is_hot_reload_enabled(&self) -> bool {
        self.hot_reload_manager
            .as_ref()
            .map(|manager| manager.is_enabled())
            .unwrap_or(false)
    }

        
    /// Manually trigger a shader reload
    ///
    /// # Arguments
    /// * `shader_path` - Path to the shader file to reload
    ///
    /// # Returns
    /// * Ok(()) if reload succeeded
    /// * Err if reload failed
    pub fn reload_shader(&self, shader_path: &str) -> Result<()> {
        if let Some(ref hot_reload) = self.hot_reload_manager {
            hot_reload.reload_shader(shader_path)?;
        } else {
            warn!("Hot reload manager not initialized");
        }
        Ok(())
    }
    
    /// Get hot reload statistics
    ///
    /// # Returns
    /// Tuple of (watched_files_count, is_enabled)
    #[allow(dead_code)]
    pub fn get_hot_reload_stats(&self) -> (usize, bool) {
        self.hot_reload_manager
            .as_ref()
            .map(|manager| manager.get_stats())
            .unwrap_or((0, false))
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
