//! HUD (Heads-Up Display) module for creating interactive toolbar interface
//!
//! This module provides a simplified HUD system with basic UI rendering
//! for creating interactive toolbars and UI elements.

pub mod toolbar;
pub mod imgui_vulkan_backend;

use crate::error::{Result, AppError};
use crate::vulkan::device::VulkanDevice;
use crate::vulkan::renderer::VulkanRenderer;
use imgui::Context;
use log::{debug, info, trace, warn};
use winit::window::Window;
use ash::vk;

/// Main HUD struct that manages the entire user interface
pub struct HUD {
    /// ImGui context for UI rendering
    pub context: Context,
    
    /// Toolbar component
    pub toolbar: toolbar::Toolbar,
    
    /// Whether HUD is enabled
    pub enabled: bool,
    
    /// Last frame time for animation
    pub last_frame_time: f32,
    
    /// Complete ImGui Vulkan backend
    pub imgui_backend: Option<imgui_vulkan_backend::ImGuiVulkanBackend>,
    
    /// Platform integration for winit
    pub platform: Option<imgui_winit_support::WinitPlatform>,
    
}

/// HUD configuration settings
#[derive(Debug, Clone)]
pub struct HUDConfig {
    /// Font size for UI elements
    #[allow(dead_code)]
    pub font_size: f32,
    
    /// Enable anti-aliasing
    #[allow(dead_code)]
    pub anti_aliasing: bool,
    
    /// Default toolbar position (top, bottom, left, right)
    #[allow(dead_code)]
    pub default_toolbar_position: ToolbarPosition,
    
    /// Toolbar background color
    #[allow(dead_code)]
    pub toolbar_background_color: [f32; 4],
    
    /// Enable keyboard shortcuts
    #[allow(dead_code)]
    pub enable_shortcuts: bool,
}

/// Toolbar position options
#[derive(Debug, Clone, Copy)]
pub enum ToolbarPosition {
    Top,
    #[allow(dead_code)]
    Bottom,
    #[allow(dead_code)]
    Left,
    #[allow(dead_code)]
    Right,
}

impl Default for HUDConfig {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            anti_aliasing: true,
            default_toolbar_position: ToolbarPosition::Top,
            toolbar_background_color: [0.1, 0.1, 0.12, 0.8],
            enable_shortcuts: true,
        }
    }
}

impl HUD {
    /// Create a new HUD instance
    ///
    /// # Arguments
    /// * `window` - The window reference
    /// * `device` - The Vulkan device
    /// * `renderer` - The Vulkan renderer
    /// * `render_pass` - The render pass for ImGui
    /// * `config` - HUD configuration
    ///
    /// # Returns
    /// A new HUD instance
    pub fn new(
        window: &Window,
        device: &VulkanDevice,
        renderer: &VulkanRenderer,
        render_pass: vk::RenderPass,
        config: HUDConfig,
    ) -> Result<Self> {
        info!("Initializing HUD system");
        
        // Create ImGui context
        let mut context = Self::create_context(&config)?;
        context.set_ini_filename(None); // Disable automatic INI saving
        
        // Initialize ImGui with winit platform
        let mut platform = imgui_winit_support::WinitPlatform::new(&mut context);
        
        // Configure ImGui display size
        let window_size = window.inner_size();
        let io = context.io_mut();
        io.display_size = [window_size.width as f32, window_size.height as f32];
        io.display_framebuffer_scale = [1.0, 1.0]; // TODO: Get actual DPI scale
        
        platform.attach_window(io, window, imgui_winit_support::HiDpiMode::Default);
        
        // Create toolbar
        let mut toolbar = toolbar::Toolbar::new(toolbar::ToolbarPosition::Top);
        
        // Set up hot reload button callback
        if let Some(_toggle_button) = toolbar.get_button("toggle_hot_reload") {
            // This will be connected to ECS world later
        }
        
        // Set up manual reload button callback
        if let Some(_reload_button) = toolbar.get_button("reload_shaders") {
            // This will be connected to ECS world later
        }
        
        // Create complete ImGui Vulkan backend
        let imgui_backend = imgui_vulkan_backend::ImGuiVulkanBackend::new(
            &device.device,
            device.physical_device,
            &renderer.instance.instance,
            render_pass,
            device.queue_families.graphics_family.unwrap(),
        ).map_err(|e| AppError::HUD(format!("Failed to create ImGui Vulkan backend: {}", e)))?;
        
        info!("HUD system initialized successfully");
        
        Ok(Self {
            context,
            toolbar,
            enabled: true,
            last_frame_time: 0.0,
            imgui_backend: Some(imgui_backend),
            platform: Some(platform),
        })
    }
    
    /// Create ImGui context with optimized settings
    /// 
    /// # Arguments
    /// * `config` - Configuration options
    /// 
    /// # Returns
    /// Configured ImGui context
    fn create_context(_config: &HUDConfig) -> Result<Context> {
        let mut context = Context::create();
        
        // Configure ImGui settings
        let io = context.io_mut();
        
        // Configure timing
        io.delta_time = 1.0 / 60.0;
        
        // Set up dark theme
        Self::setup_dark_theme(&mut context);
        
        Ok(context)
    }
    
    /// Setup a dark theme similar to Blender
    fn setup_dark_theme(context: &mut Context) {
        let style = context.style_mut();
        
        // Professional dark theme with better contrast and appearance
        style.colors[imgui::StyleColor::Text as usize] = [0.92, 0.92, 0.92, 1.00];
        style.colors[imgui::StyleColor::TextDisabled as usize] = [0.50, 0.50, 0.50, 1.00];
        style.colors[imgui::StyleColor::WindowBg as usize] = [0.13, 0.13, 0.15, 0.90];
        style.colors[imgui::StyleColor::ChildBg as usize] = [0.13, 0.13, 0.15, 0.90];
        style.colors[imgui::StyleColor::PopupBg as usize] = [0.13, 0.13, 0.15, 0.95];
        style.colors[imgui::StyleColor::Border as usize] = [0.25, 0.25, 0.30, 0.50];
        style.colors[imgui::StyleColor::BorderShadow as usize] = [0.00, 0.00, 0.00, 0.00];
        style.colors[imgui::StyleColor::FrameBg as usize] = [0.16, 0.16, 0.18, 1.00];
        style.colors[imgui::StyleColor::FrameBgHovered as usize] = [0.20, 0.20, 0.23, 1.00];
        style.colors[imgui::StyleColor::FrameBgActive as usize] = [0.24, 0.24, 0.28, 1.00];
        style.colors[imgui::StyleColor::TitleBg as usize] = [0.16, 0.16, 0.18, 1.00];
        style.colors[imgui::StyleColor::TitleBgActive as usize] = [0.20, 0.20, 0.23, 1.00];
        style.colors[imgui::StyleColor::TitleBgCollapsed as usize] = [0.16, 0.16, 0.18, 1.00];
        style.colors[imgui::StyleColor::MenuBarBg as usize] = [0.16, 0.16, 0.18, 1.00];
        style.colors[imgui::StyleColor::ScrollbarBg as usize] = [0.10, 0.10, 0.12, 1.00];
        style.colors[imgui::StyleColor::ScrollbarGrab as usize] = [0.20, 0.20, 0.23, 1.00];
        style.colors[imgui::StyleColor::ScrollbarGrabHovered as usize] = [0.24, 0.24, 0.28, 1.00];
        style.colors[imgui::StyleColor::ScrollbarGrabActive as usize] = [0.28, 0.28, 0.32, 1.00];
        style.colors[imgui::StyleColor::CheckMark as usize] = [0.26, 0.59, 0.98, 1.00];
        style.colors[imgui::StyleColor::SliderGrab as usize] = [0.26, 0.59, 0.98, 1.00];
        style.colors[imgui::StyleColor::SliderGrabActive as usize] = [0.46, 0.79, 0.98, 1.00];
        style.colors[imgui::StyleColor::Button as usize] = [0.35, 0.35, 0.40, 1.00];
        style.colors[imgui::StyleColor::ButtonHovered as usize] = [0.45, 0.45, 0.50, 1.00];
        style.colors[imgui::StyleColor::ButtonActive as usize] = [0.55, 0.55, 0.60, 1.00];
        style.colors[imgui::StyleColor::Header as usize] = [0.20, 0.20, 0.23, 1.00];
        style.colors[imgui::StyleColor::HeaderHovered as usize] = [0.26, 0.26, 0.30, 1.00];
        style.colors[imgui::StyleColor::HeaderActive as usize] = [0.32, 0.32, 0.36, 1.00];
        style.colors[imgui::StyleColor::Separator as usize] = [0.20, 0.20, 0.23, 1.00];
        style.colors[imgui::StyleColor::SeparatorHovered as usize] = [0.26, 0.26, 0.30, 1.00];
        style.colors[imgui::StyleColor::SeparatorActive as usize] = [0.32, 0.32, 0.36, 1.00];
        style.colors[imgui::StyleColor::ResizeGrip as usize] = [0.26, 0.59, 0.98, 0.20];
        style.colors[imgui::StyleColor::ResizeGripHovered as usize] = [0.26, 0.59, 0.98, 0.67];
        style.colors[imgui::StyleColor::ResizeGripActive as usize] = [0.46, 0.79, 0.98, 0.95];
        style.colors[imgui::StyleColor::Tab as usize] = [0.20, 0.20, 0.23, 1.00];
        style.colors[imgui::StyleColor::TabHovered as usize] = [0.26, 0.26, 0.30, 1.00];
        style.colors[imgui::StyleColor::TabActive as usize] = [0.32, 0.32, 0.36, 1.00];
        style.colors[imgui::StyleColor::TabUnfocused as usize] = [0.20, 0.20, 0.23, 1.00];
        style.colors[imgui::StyleColor::TabUnfocusedActive as usize] = [0.26, 0.26, 0.30, 1.00];
        style.colors[imgui::StyleColor::PlotLines as usize] = [0.61, 0.61, 0.61, 1.00];
        style.colors[imgui::StyleColor::PlotLinesHovered as usize] = [1.00, 1.00, 1.00, 1.00];
        style.colors[imgui::StyleColor::PlotHistogram as usize] = [0.90, 0.70, 0.00, 1.00];
        style.colors[imgui::StyleColor::PlotHistogramHovered as usize] = [1.00, 0.60, 0.00, 1.00];
        style.colors[imgui::StyleColor::TextSelectedBg as usize] = [0.26, 0.59, 0.98, 0.35];
        style.colors[imgui::StyleColor::DragDropTarget as usize] = [0.26, 0.59, 0.98, 0.95];
        style.colors[imgui::StyleColor::NavHighlight as usize] = [0.26, 0.59, 0.98, 0.80];
        style.colors[imgui::StyleColor::NavWindowingHighlight as usize] = [1.00, 1.00, 1.00, 0.70];
        style.colors[imgui::StyleColor::NavWindowingDimBg as usize] = [0.20, 0.20, 0.20, 0.20];
        style.colors[imgui::StyleColor::ModalWindowDimBg as usize] = [0.20, 0.20, 0.20, 0.35];
        
        // Improve styling for better toolbar appearance
        style.window_padding = [12.0, 8.0];
        style.window_rounding = 0.0;
        style.window_border_size = 0.0;
        style.frame_padding = [8.0, 6.0];
        style.frame_rounding = 4.0;
        style.frame_border_size = 0.0;
        style.item_spacing = [12.0, 8.0];
        style.item_inner_spacing = [8.0, 6.0];
        style.indent_spacing = 21.0;
        style.scrollbar_size = 14.0;
        style.scrollbar_rounding = 2.0;
        style.grab_min_size = 10.0;
        style.grab_rounding = 2.0;
        style.tab_rounding = 4.0;
        style.button_text_align = [0.5, 0.5];
        style.display_window_padding = [8.0, 8.0];
        style.display_safe_area_padding = [4.0, 4.0];
        style.anti_aliased_lines = true;
        style.anti_aliased_fill = true;
        style.curve_tessellation_tol = 1.25;
        
        // Improve font rendering
        style.window_min_size = [200.0, 50.0];
    }
    
    /// Handle window resize
    /// 
    /// # Arguments
    /// * `extent` - New extent
    pub fn handle_resize(&mut self, extent: vk::Extent2D) {
        debug!("HUD resize handled for extent: {}x{}", extent.width, extent.height);
    }
    
    
    /// Update the HUD state (called each frame before rendering)

    /// Check if manual reload button was clicked
    pub fn was_reload_button_clicked(&self) -> bool {
        self.toolbar.was_button_clicked("reload_shaders")
    }

    /// Check if hot reload checkbox was toggled
    pub fn was_hot_reload_toggled(&self) -> Option<bool> {
        self.toolbar.was_hot_reload_toggled()
    }

    /// Update HUD state and animations
    ///
    /// # Arguments
    /// * `window` - Current window for input handling
    /// * `delta_time` - Time since last frame
    pub fn update(&mut self, window: &winit::window::Window, delta_time: f32) {
        if !self.enabled {
            return;
        }
        
        self.last_frame_time = delta_time;
        
        // Update platform integration
        if let Some(platform) = &mut self.platform {
            let io = self.context.io_mut();
            
            // Handle new frame event
            platform.handle_event(io, window, &winit::event::Event::<()>::NewEvents(
                winit::event::StartCause::Init
            ));
            
            // Update display size
            let window_size = window.inner_size();
            io.display_size = [window_size.width as f32, window_size.height as f32];
            io.display_framebuffer_scale = [1.0, 1.0]; // TODO: Get actual DPI scale
            
            // Enable mouse input
            io.backend_flags |= imgui::BackendFlags::HAS_MOUSE_CURSORS;
            io.backend_flags |= imgui::BackendFlags::HAS_SET_MOUSE_POS;
        }
        
        // Update toolbar
        self.toolbar.update(delta_time);
        
        // Update context
        self.context.io_mut().delta_time = delta_time;
    }

        
    
    /// Render the HUD
    ///
    /// # Arguments
    /// * `command_buffer` - Command buffer to record commands
    /// * `extent` - Current render extent
    pub fn render(
        &mut self,
        command_buffer: vk::CommandBuffer,
        extent: vk::Extent2D,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        // Update ImGui display size
        let io = self.context.io_mut();
        io.display_size = [extent.width as f32, extent.height as f32];
        io.display_framebuffer_scale = [1.0, 1.0]; // TODO: Get actual DPI scale
        
        // Create a new ImGui frame
        let ui = self.context.frame();
        
        // Render the toolbar - this creates the UI elements
        // Note: In a full implementation, you'd pass ECS world reference here
        self.toolbar.render(&ui);
        
        // Get the draw data and render it using Vulkan backend
        let draw_data = self.context.render();
        
        // Render ImGui using complete Vulkan backend
        if let Some(imgui_backend) = &mut self.imgui_backend {
            imgui_backend.render(draw_data, command_buffer)?;
            debug!("ImGui rendered successfully with {} draw lists", draw_data.draw_lists().count());
            
            // Don't clean up buffers after each frame - they should persist until next frame
            // The buffers will be cleaned up when new ones are created or during shutdown
        } else {
            // Fallback: at least log that we're trying to render
            warn!("No ImGui renderer available for HUD - toolbar created but not visible");
        }
        
        // Note: Overlay renderer removed - we now use real ImGui rendering only
        
        trace!("HUD rendering completed");
        Ok(())
    }
    
    /// Enable or disable the HUD
    ///
    /// # Arguments
    /// * `enabled` - Whether HUD should be enabled
    #[allow(dead_code)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        debug!("HUD {}", if enabled { "enabled" } else { "disabled" });
    }
    
    /// Initialize font texture for ImGui
    pub fn init_font_texture(&mut self) -> Result<()> {
        debug!("Initializing font texture for ImGui");
        
        // Build font atlas with better fonts
        let fonts = self.context.fonts();
        
        // Configure font for better readability
        let mut font_config = imgui::FontConfig::default();
        font_config.size_pixels = 16.0; // Slightly larger for better readability
        font_config.oversample_h = 2; // Better horizontal rendering
        font_config.oversample_v = 1; // Better vertical rendering
        font_config.pixel_snap_h = true; // Crisp text rendering
        
        // Try to add a better font - you can customize this
        let font_sources = vec![
            // Option 1: Try to load a system font using TtfData (we'll read the file at runtime)
            // For now, we'll use the default font with better configuration
            imgui::FontSource::DefaultFontData {
                config: Some(font_config),
            },
        ];
        
        fonts.add_font(&font_sources);
        
        // Get font texture data
        let font_texture = fonts.build_rgba32_texture();
        
        // Upload font texture to GPU
        if let Some(imgui_backend) = &mut self.imgui_backend {
            imgui_backend.create_font_texture(font_texture.width, font_texture.height)?;
            
            // Upload the actual font data
            imgui_backend.upload_font_data(font_texture.width, font_texture.height, font_texture.data)?;
            debug!("Font texture uploaded with size {}x{}", font_texture.width, font_texture.height);
        }
        Ok(())
    }
    
    /// Check if HUD is enabled
    /// 
    /// # Returns
    /// True if HUD is enabled
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Get toolbar reference
    /// 
    /// # Returns
    /// Reference to toolbar
    #[allow(dead_code)]
    pub fn toolbar(&self) -> &toolbar::Toolbar {
        &self.toolbar
    }
    
    /// Get mutable toolbar reference
    ///
    /// # Returns
    /// Mutable reference to toolbar
    #[allow(dead_code)]
    pub fn toolbar_mut(&mut self) -> &mut toolbar::Toolbar {
        &mut self.toolbar
    }
    
    /// Get mutable ImGui IO for direct input handling
    ///
    /// # Returns
    /// Mutable reference to ImGui IO
    pub fn context_mut(&mut self) -> &mut imgui::Io {
        self.context.io_mut()
    }

    /// Set up hot reload callbacks after ECS world is available
    ///
    /// # Arguments
    /// * `ecs_world` - Reference to ECS world for hot reload functionality
    #[allow(dead_code)]
    pub fn setup_hot_reload_callbacks(&mut self, ecs_world: &mut crate::ecs::world::ECSWorld) {
        // Set up hot reload toggle button
        if let Some(toggle_button) = self.toolbar.get_button_mut("toggle_hot_reload") {
            let is_enabled = ecs_world.is_hot_reload_enabled();
            toggle_button.is_active = is_enabled;

            // Create a global flag to track the last known state (simpler approach)
            // Note: This is a temporary solution - in a real app you'd want a proper event system
        }
        
        // Set up manual reload button
        if let Some(reload_button) = self.toolbar.get_button_mut("reload_shaders") {
            let ecs_world_ptr = ecs_world as *mut crate::ecs::world::ECSWorld;
            reload_button.action = Some(Box::new(move || {
                // Safe to access because we know the ECS world lives longer than the callback
                let ecs_world = unsafe { &mut *ecs_world_ptr };
                
                // Try to reload all shader files
                let shader_files = ["shaders/sdf.vert", "shaders/sdf.frag", "shaders/imgui.vert", "shaders/imgui.frag"];
                for shader_file in &shader_files {
                    if let Err(e) = ecs_world.reload_shader(shader_file) {
                        log::error!("Failed to reload shader {}: {}", shader_file, e);
                    } else {
                        log::info!("Manually reloaded shader: {}", shader_file);
                    }
                }
            }));
        }
        
        log::info!("Hot reload callbacks set up in toolbar");
    }
    
}

impl Drop for HUD {
    fn drop(&mut self) {
        info!("Destroying HUD system");
        
        // Explicitly clean up the ImGui Vulkan backend to ensure buffers are destroyed
        // before the Vulkan device is destroyed
        if let Some(ref mut backend) = self.imgui_backend {
            debug!("Cleaning up ImGui Vulkan backend");
            // The backend.cleanup() method now handles device wait idle internally
            backend.cleanup();
        }
        
        // Clear the backend reference
        self.imgui_backend = None;
        
        debug!("HUD system destroyed");
    }
}
