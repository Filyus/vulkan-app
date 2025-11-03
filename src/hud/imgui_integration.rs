//! ImGui integration module for Vulkan rendering
//! 
//! This module provides specialized integration between ImGui and Vulkan,
//! handling command buffer recording, texture management, and rendering pipeline setup.

use ash::vk;
use ash::Device;
use imgui::{Context, DrawData, Ui};
use imgui_ash::AshRenderer;
use crate::error::{Result, VulkanError};
use log::{debug, error, trace};

/// Enhanced ImGui renderer with additional features
pub struct ImGuiIntegration {
    /// Core Ash renderer
    renderer: AshRenderer,
    
    /// Last rendered extent
    last_extent: vk::Extent2D,
    
    /// Whether rendering is enabled
    is_enabled: bool,
    
    /// Render scale for high-DPI displays
    render_scale: f32,
}

/// ImGui render configuration
#[derive(Debug, Clone)]
pub struct ImGuiConfig {
    /// Enable anti-aliasing
    pub anti_aliasing: bool,
    
    /// Render scale (1.0 = normal, 2.0 = retina/hi-dpi)
    pub render_scale: f32,
    
    /// Alpha blending for transparent elements
    pub alpha_blending: bool,
    
    /// Enable keyboard navigation
    pub keyboard_nav: bool,
}

impl Default for ImGuiConfig {
    fn default() -> Self {
        Self {
            anti_aliasing: true,
            render_scale: 1.0,
            alpha_blending: true,
            keyboard_nav: true,
        }
    }
}

impl ImGuiIntegration {
    /// Create a new ImGui integration
    /// 
    /// # Arguments
    /// * `instance` - Vulkan instance
    /// * `device` - Vulkan device  
    /// * `swapchain_format` - Format of swapchain images
    /// * `config` - Configuration options
    /// 
    /// # Returns
    /// New ImGuiIntegration instance
    pub fn new(
        instance: &ash::Instance,
        device: &Device,
        swapchain_format: vk::Format,
        config: ImGuiConfig,
    ) -> Result<Self> {
        debug!("Initializing ImGui integration");
        
        let renderer = AshRenderer::new(instance, device, swapchain_format)
            .map_err(|e| VulkanError::Rendering(format!("Failed to create Ash renderer: {}", e)))?;
        
        debug!("ImGui integration initialized successfully");
        
        Ok(Self {
            renderer,
            last_extent: vk::Extent2D::default(),
            is_enabled: true,
            render_scale: config.render_scale,
        })
    }
    
    /// Render ImGui UI to a command buffer
    /// 
    /// # Arguments
    /// * `context` - ImGui context containing the UI to render
    /// * `device` - Vulkan device
    /// * `command_buffer` - Command buffer to record to
    /// * `extent` - Current render extent
    /// 
    /// # Returns
    /// Ok(()) on success, error on failure
    pub fn render(
        &mut self,
        context: &mut Context,
        device: &Device,
        command_buffer: vk::CommandBuffer,
        extent: vk::Extent2D,
    ) -> Result<()> {
        if !self.is_enabled {
            return Ok(());
        }
        
        trace!("Rendering ImGui UI to command buffer");
        
        // Update render scale for hi-dpi displays
        self.update_render_scale(extent);
        
        // Render using AshRenderer
        self.renderer
            .render(context, device, command_buffer, extent)
            .map_err(|e| VulkanError::Rendering(format!("ImGui render failed: {}", e)))?;
        
        self.last_extent = extent;
        trace!("ImGui rendering completed");
        Ok(())
    }
    
    /// Update render scale based on extent and DPI
    fn update_render_scale(&mut self, extent: vk::Extent2D) {
        // Simple heuristic for high-DPI detection
        // In a real implementation, you'd query actual DPI from the window
        let width_dpi = extent.width as f32;
        let is_high_dpi = width_dpi > 1920.0;
        
        self.render_scale = if is_high_dpi { 2.0 } else { 1.0 };
    }
    
    /// Handle window resize
    /// 
    /// # Arguments
    /// * `extent` - New extent
    pub fn handle_resize(&mut self, extent: vk::Extent2D) {
        debug!("ImGui integration handling resize to {}x{}", extent.width, extent.height);
        self.last_extent = extent;
        self.update_render_scale(extent);
    }
    
    /// Enable or disable rendering
    /// 
    /// # Arguments
    /// * `enabled` - Whether rendering should be enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
        debug!("ImGui rendering {}", if enabled { "enabled" } else { "disabled" });
    }
    
    /// Check if rendering is enabled
    pub fn is_rendering_enabled(&self) -> bool {
        self.is_enabled
    }
    
    /// Get current render scale
    pub fn render_scale(&self) -> f32 {
        self.render_scale
    }
    
    /// Create ImGui context with optimized settings
    /// 
    /// # Arguments
    /// * `config` - Configuration options
    /// 
    /// # Returns
    /// Configured ImGui context
    pub fn create_context(config: &ImGuiConfig) -> Context {
        let mut context = Context::create();
        
        // Disable automatic INI file saving
        context.set_ini_filename(None);
        
        // Configure ImGui settings
        let io = context.io_mut();
        
        // Enable keyboard and mouse
        io.backend_flags |= imgui::BackendFlags::HAS_KEYBOARD;
        io.backend_flags |= imgui::BackendFlags::HAS_MOUSE;
        io.backend_flags |= imgui::BackendFlags::HAS_GAMEPAD;
        
        // Configure mouse behavior for toolbar
        io.mouse_double_click_time = 0.4;
        io.mouse_double_click_max_dist = 6.0;
        
        // Configure timing
        io.delta_time = 1.0 / 60.0;
        
        // Enable anti-aliasing if configured
        if config.anti_aliasing {
            io.mouse_draw_cursor = true;
        }
        
        // Set up dark theme
        Self::setup_dark_theme(&mut context);
        
        context
    }
    
    /// Setup a dark theme similar to Blender
    fn setup_dark_theme(context: &mut Context) {
        let style = context.style_mut();
        
        // Dark theme colors
        style.colors = [
            (imgui::StyleColor::Text, [0.90, 0.90, 0.90, 1.00]),
            (imgui::StyleColor::TextDisabled, [0.60, 0.60, 0.60, 1.00]),
            (imgui::StyleColor::WindowBg, [0.10, 0.10, 0.12, 0.80]),
            (imgui::StyleColor::ChildBg, [0.10, 0.10, 0.12, 0.80]),
            (imgui::StyleColor::PopupBg, [0.10, 0.10, 0.12, 0.95]),
            (imgui::StyleColor::Border, [0.20, 0.20, 0.25, 1.00]),
            (imgui::StyleColor::BorderShadow, [0.00, 0.00, 0.00, 0.00]),
            (imgui::StyleColor::FrameBg, [0.20, 0.20, 0.25, 1.00]),
            (imgui::StyleColor::FrameBgHovered, [0.25, 0.25, 0.30, 1.00]),
            (imgui::StyleColor::FrameBgActive, [0.30, 0.30, 0.35, 1.00]),
            (imgui::StyleColor::TitleBg, [0.10, 0.10, 0.12, 0.80]),
            (imgui::StyleColor::TitleBgActive, [0.15, 0.15, 0.18, 0.80]),
            (imgui::StyleColor::TitleBgCollapsed, [0.10, 0.10, 0.12, 0.80]),
            (imgui::StyleColor::MenuBarBg, [0.15, 0.15, 0.18, 0.80]),
            (imgui::StyleColor::ScrollbarBg, [0.15, 0.15, 0.18, 0.80]),
            (imgui::StyleColor::ScrollbarGrab, [0.25, 0.25, 0.30, 1.00]),
            (imgui::StyleColor::ScrollbarGrabHovered, [0.30, 0.30, 0.35, 1.00]),
            (imgui::StyleColor::ScrollbarGrabActive, [0.35, 0.35, 0.40, 1.00]),
            (imgui::StyleColor::CheckMark, [0.40, 0.70, 0.90, 1.00]),
            (imgui::StyleColor::SliderGrab, [0.40, 0.70, 0.90, 1.00]),
            (imgui::StyleColor::SliderGrabActive, [0.50, 0.80, 0.95, 1.00]),
            (imgui::StyleColor::Button, [0.20, 0.20, 0.25, 1.00]),
            (imgui::StyleColor::ButtonHovered, [0.25, 0.25, 0.30, 1.00]),
            (imgui::StyleColor::ButtonActive, [0.30, 0.30, 0.35, 1.00]),
            (imgui::StyleColor::Header, [0.20, 0.20, 0.25, 1.00]),
            (imgui::StyleColor::HeaderHovered, [0.25, 0.25, 0.30, 1.00]),
            (imgui::StyleColor::HeaderActive, [0.30, 0.30, 0.35, 1.00]),
            (imgui::StyleColor::Separator, [0.20, 0.20, 0.25, 1.00]),
            (imgui::StyleColor::SeparatorHovered, [0.25, 0.25, 0.30, 1.00]),
            (imgui::StyleColor::SeparatorActive, [0.30, 0.30, 0.35, 1.00]),
            (imgui::StyleColor::ResizeGrip, [0.20, 0.20, 0.25, 1.00]),
            (imgui::StyleColor::ResizeGripHovered, [0.25, 0.25, 0.30, 1.00]),
            (imgui::StyleColor::ResizeGripActive, [0.30, 0.30, 0.35, 1.00]),
            (imgui::StyleColor::Tab, [0.15, 0.15, 0.18, 0.80]),
            (imgui::StyleColor::TabHovered, [0.20, 0.20, 0.25, 1.00]),
            (imgui::StyleColor::TabActive, [0.25, 0.25, 0.30, 1.00]),
            (imgui::StyleColor::TabUnfocused, [0.15, 0.15, 0.18, 0.80]),
            (imgui::StyleColor::TabUnfocusedActive, [0.20, 0.20, 0.25, 1.00]),
            (imgui::StyleColor::PlotLines, [0.40, 0.70, 0.90, 1.00]),
            (imgui::StyleColor::PlotLinesHovered, [0.50, 0.80, 0.95, 1.00]),
            (imgui::StyleColor::PlotHistogram, [0.40, 0.70, 0.90, 1.00]),
            (imgui::StyleColor::PlotHistogramHovered, [0.50, 0.80, 0.95, 1.00]),
            (imgui::StyleColor::TextSelectedBg, [0.30, 0.30, 0.35, 1.00]),
            (imgui::StyleColor::DragDropTarget, [0.40, 0.70, 0.90, 1.00]),
            (imgui::StyleColor::NavHighlight, [0.40, 0.70, 0.90, 1.00]),
            (imgui::StyleColor::NavWindowingHighlight, [0.40, 0.70, 0.90, 1.00]),
            (imgui::StyleColor::NavWindowingDimBg, [0.10, 0.10, 0.12, 0.50]),
            (imgui::StyleColor::ModalWindowDimBg, [0.10, 0.10, 0.12, 0.50]),
        ].into_iter().collect();
        
        // Configure spacing and sizing
        style.frame_padding = [4.0, 3.0];
        style.item_spacing = [8.0, 4.0];
        style.indent_spacing = 12.0;
        style.scrollbar_size = 14.0;
        style.grab_min_size = 12.0;
        
        // Rounding
        style.window_rounding = 4.0;
        style.child_rounding = 4.0;
        style.frame_rounding = 3.0;
        style.scrollbar_rounding = 4.0;
        style.tabs_rounding = 0.0;
        
        // Borders
        style.window_border_size = 1.0;
        style.child_border_size = 1.0;
        style.frame_border_size = 1.0;
        style.popup_border_size = 1.0;
        style.tabs_border_size = 0.0;
        style.separator_text_border_size = 0.0;
    }
    
    /// Get the underlying AshRenderer reference
    pub fn renderer(&self) -> &AshRenderer {
        &self.renderer
    }
    
    /// Get mutable reference to underlying AshRenderer
    pub fn renderer_mut(&mut self) -> &mut AshRenderer {
        &mut self.renderer
    }
}

impl Drop for ImGuiIntegration {
    fn drop(&mut self) {
        debug!("Destroying ImGui integration");
        // AshRenderer cleanup is handled by its own Drop implementation
    }
}