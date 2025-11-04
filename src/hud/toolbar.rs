//! Toolbar component for creating interactive toolbar interface
//!
//! This module provides a simplified toolbar system with buttons, icons,
//! tooltips, and interactive elements for a professional UI experience.

use imgui::{Ui, Key};
use std::time::Instant;
use log::{info, debug};

/// Button interaction states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Active,
    Disabled,
}

/// Toolbar button with enhanced interactivity
pub struct ToolbarButton {
    /// Button identifier
    #[allow(dead_code)]
    pub id: String,
    
    /// Button icon (text representation)
    pub icon: &'static str,
    
    /// Button tooltip
    pub tooltip: &'static str,
    
    /// Whether button is active/pressed
    pub is_active: bool,
    
    /// Whether button is enabled
    pub is_enabled: bool,
    
    /// Button action callback
    pub action: Option<Box<dyn Fn() + 'static>>,
    
    /// Last interaction time (for animation)
    pub last_interaction: Option<Instant>,
    
    /// Current button state
    pub state: ButtonState,
    
    /// Hover animation progress (0.0 to 1.0)
    pub hover_progress: f32,
    
    /// Click animation progress (0.0 to 1.0)
    pub click_animation: f32,
    
    /// Button color theme
    pub color_theme: ButtonColorTheme,
}

/// Color theme for buttons
#[derive(Debug, Clone, Copy)]
pub struct ButtonColorTheme {
    /// Normal state color
    pub normal: [f32; 4],
    /// Hovered state color
    pub hovered: [f32; 4],
    /// Active state color
    pub active: [f32; 4],
    /// Disabled state color
    pub disabled: [f32; 4],
    /// Text color
    pub text: [f32; 4],
}

impl Default for ButtonColorTheme {
    fn default() -> Self {
        Self {
            normal: [0.2, 0.25, 0.35, 1.0],      // Blue-ish base
            hovered: [0.3, 0.35, 0.45, 1.0],     // Lighter blue
            active: [0.25, 0.3, 0.4, 1.0],      // Slightly darker blue (same hue)
            disabled: [0.15, 0.15, 0.2, 0.5],   // Desaturated blue
            text: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Toolbar group containing related buttons
pub struct ToolbarGroup {
    /// Group name
    pub name: &'static str,
    
    /// Group buttons
    pub buttons: Vec<ToolbarButton>,
    
    /// Whether group is collapsible
    #[allow(dead_code)]
    pub collapsible: bool,
    
    /// Whether group is collapsed
    #[allow(dead_code)]
    pub is_collapsed: bool,
}

/// Toolbar position and layout
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

/// Main toolbar component
pub struct Toolbar {
    /// Toolbar position
    #[allow(dead_code)]
    pub position: ToolbarPosition,
    
    /// Toolbar groups
    pub groups: Vec<ToolbarGroup>,
    
    /// Whether toolbar is visible
    pub is_visible: bool,
    
    /// Whether toolbar is floating
    #[allow(dead_code)]
    pub is_floating: bool,
    
    /// Toolbar background alpha
    #[allow(dead_code)]
    pub background_alpha: f32,
    
    /// Animation time accumulator
    pub animation_time: f32,
    
    /// Whether to show labels
    #[allow(dead_code)]
    pub show_labels: bool,
}

impl Toolbar {
    /// Create a new toolbar
    pub fn new(position: ToolbarPosition) -> Self {
        Self {
            position,
            groups: Self::create_default_groups(),
            is_visible: true,
            is_floating: false,
            background_alpha: 0.8,
            animation_time: 0.0,
            show_labels: true,
        }
    }
    
    /// Create default toolbar groups
    fn create_default_groups() -> Vec<ToolbarGroup> {
        vec![
            // Add Objects
            ToolbarGroup {
                name: "",
                buttons: vec![
                    ToolbarButton {
                        id: "add_sphere".to_string(),
                        icon: "Add Sphere",
                        tooltip: "Add Sphere to scene",
                        is_active: false,
                        is_enabled: true,
                        action: Some(Box::new(|| {
                            info!("Add sphere action triggered");
                        })),
                        last_interaction: None,
                        state: ButtonState::Normal,
                        hover_progress: 0.0,
                        click_animation: 0.0,
                        color_theme: ButtonColorTheme::default(),
                    },
                    ToolbarButton {
                        id: "add_box".to_string(),
                        icon: "Add Box",
                        tooltip: "Add Box to scene",
                        is_active: false,
                        is_enabled: true,
                        action: Some(Box::new(|| {
                            info!("Add box action triggered");
                        })),
                        last_interaction: None,
                        state: ButtonState::Normal,
                        hover_progress: 0.0,
                        click_animation: 0.0,
                        color_theme: ButtonColorTheme::default(),
                    },
                ],
                collapsible: false,
                is_collapsed: false,
            },
            // Hot Reload Controls
            ToolbarGroup {
                name: "",
                buttons: vec![
                    ToolbarButton {
                        id: "toggle_hot_reload".to_string(),
                        icon: "🔥 Hot Reload",
                        tooltip: "Toggle hot shader reload (F2)",
                        is_active: false,
                        is_enabled: true,
                        action: None, // Will be set by HUD
                        last_interaction: None,
                        state: ButtonState::Normal,
                        hover_progress: 0.0,
                        click_animation: 0.0,
                        color_theme: ButtonColorTheme {
                            normal: [0.3, 0.2, 0.4, 1.0],      // Purple base
                            hovered: [0.4, 0.3, 0.5, 1.0],     // Lighter purple
                            active: [0.5, 0.4, 0.6, 1.0],      // Bright purple
                            disabled: [0.2, 0.15, 0.3, 0.5],   // Desaturated purple
                            text: [1.0, 1.0, 1.0, 1.0],
                        },
                    },
                    ToolbarButton {
                        id: "reload_shaders".to_string(),
                        icon: "🔄 Reload",
                        tooltip: "Manual shader reload (F3)",
                        is_active: false,
                        is_enabled: true,
                        action: None, // Will be set by HUD
                        last_interaction: None,
                        state: ButtonState::Normal,
                        hover_progress: 0.0,
                        click_animation: 0.0,
                        color_theme: ButtonColorTheme {
                            normal: [0.2, 0.4, 0.3, 1.0],      // Green base
                            hovered: [0.3, 0.5, 0.4, 1.0],     // Lighter green
                            active: [0.4, 0.6, 0.5, 1.0],      // Bright green
                            disabled: [0.15, 0.3, 0.2, 0.5],   // Desaturated green
                            text: [1.0, 1.0, 1.0, 1.0],
                        },
                    },
                ],
                collapsible: false,
                is_collapsed: false,
            },
        ]
    }
    
    /// Update toolbar state (called each frame)
    pub fn update(&mut self, delta_time: f32) {
        self.animation_time += delta_time;
        
        // Update button interactions and animations
        for group in &mut self.groups {
            for button in &mut group.buttons {
                Self::update_button_animations(button, delta_time);
                
                if let Some(last_interaction) = button.last_interaction {
                    // Reset button state after animation
                    if last_interaction.elapsed().as_secs_f32() > 0.2 {
                        button.last_interaction = None;
                    }
                }
            }
        }
    }
    
    /// Update button animations and state transitions
    fn update_button_animations(button: &mut ToolbarButton, delta_time: f32) {
        const HOVER_SPEED: f32 = 8.0;
        const CLICK_SPEED: f32 = 12.0;
        
        // Update hover animation
        let target_hover = match button.state {
            ButtonState::Hovered | ButtonState::Active => 1.0,
            _ => 0.0,
        };
        
        button.hover_progress += (target_hover - button.hover_progress) * HOVER_SPEED * delta_time;
        button.hover_progress = button.hover_progress.clamp(0.0, 1.0);
        
        // Update click animation
        if button.click_animation > 0.0 {
            button.click_animation -= CLICK_SPEED * delta_time;
            button.click_animation = button.click_animation.max(0.0);
        }
        
        // Update button state based on enabled status
        if !button.is_enabled {
            button.state = ButtonState::Disabled;
        } else if button.is_active {
            button.state = ButtonState::Active;
        } else if button.state == ButtonState::Disabled {
            button.state = ButtonState::Normal;
        }
    }
    
    /// Render the toolbar using ImGui
    pub fn render(&mut self, ui: &Ui) {
        if !self.is_visible {
            return;
        }
        
        // Create a professional toolbar window
        let window_flags = imgui::WindowFlags::NO_DECORATION
            | imgui::WindowFlags::NO_RESIZE
            | imgui::WindowFlags::NO_COLLAPSE
            | imgui::WindowFlags::ALWAYS_AUTO_RESIZE
            | imgui::WindowFlags::NO_MOVE
            | imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS;
        
        // Create toolbar window with better positioning and styling
        let window = ui.window("##Toolbar")
            .position([0.0, 0.0], imgui::Condition::Always)
            .size([ui.io().display_size[0], 60.0], imgui::Condition::Always)
            .bg_alpha(0.95)
            .flags(window_flags);
        
        if let Some(_token) = window.begin() {
            // Calculate vertical center position for the toolbar content
            let window_height = 60.0; // Toolbar height
            let content_height = 28.0; // Approximate button height
            let vertical_center = (window_height - content_height) / 2.0;
            
            // Position content at vertical center
            ui.set_cursor_pos([8.0, vertical_center]);
            
            // Render toolbar content with improved styling
            self.render_toolbar_content(ui);
        }
        
        // Render tooltips and handle shortcuts
        self.render_shortcuts(ui);
    }
    
    /// Render the actual toolbar content inside the window
    fn render_toolbar_content(&mut self, ui: &Ui) {
        // Use a group for better layout control
        ui.group(|| {
            // Render groups with their buttons in a clean horizontal layout
            let group_count = self.groups.len();
            for i in 0..group_count {
                // Render buttons in this group
                let button_count = self.groups[i].buttons.len();
                for j in 0..button_count {
                    self.render_button_by_indices(ui, i, j);
                    if j < button_count - 1 {
                        ui.same_line();
                    }
                }
            }
        });
    }
    
    /// Render toolbar at the top
    #[allow(dead_code)]
    fn render_top_toolbar(&mut self, ui: &Ui) {
        // Create a simple window at the top
        ui.set_cursor_pos([0.0, 0.0]);
        
        // Simple background using default window styling
        // The actual styling is handled by ImGui theme
        
        ui.set_cursor_pos([4.0, 4.0]);
        
        // Render groups with their buttons
        let group_count = self.groups.len();
        for i in 0..group_count {
            if i > 0 {
                ui.separator();
                ui.same_line();
            }
            
            // Get group name and buttons without borrowing issues
            let group_name = self.groups[i].name;
            ui.text(group_name);
            ui.same_line();
            
            // Render buttons in this group
            let button_count = self.groups[i].buttons.len();
            for j in 0..button_count {
                ui.same_line();
                self.render_button_by_indices(ui, i, j);
            }
        }
    }
    
    /// Render button by indices to avoid borrowing issues
    fn render_button_by_indices(&mut self, ui: &Ui, group_idx: usize, button_idx: usize) {
        if let Some(button) = self.groups.get_mut(group_idx).and_then(|g| g.buttons.get_mut(button_idx)) {
            // Use the full text as button label
            let button_label = button.icon.to_string();
            
            // Calculate button size based on text content
            let text_width = ui.calc_text_size(&button_label)[0] + 20.0; // Add padding
            let base_button_size = [text_width, 28.0];
            
            // Use consistent button size (no click animation scaling)
            let button_size = base_button_size;
            
            // Calculate interpolated colors based on state and animations
            let button_color = Self::calculate_button_color(button);
            let hover_color = Self::calculate_hover_color(button);
            let active_color = Self::calculate_active_color(button);
            
            // Enhanced styling with animations
            let _style_token = ui.push_style_color(imgui::StyleColor::Button, button_color);
            let _style_token2 = ui.push_style_color(imgui::StyleColor::ButtonHovered, hover_color);
            let _style_token3 = ui.push_style_color(imgui::StyleColor::ButtonActive, active_color);
            let _style_token4 = ui.push_style_color(imgui::StyleColor::Text, button.color_theme.text);
            
            // Dynamic rounding based on hover state
            let rounding = 3.0 + button.hover_progress * 2.0;
            let _style_token5 = ui.push_style_var(imgui::StyleVar::FrameRounding(rounding));
            
            // Enhanced padding with hover effect
            let padding_multiplier = 1.0 + button.hover_progress * 0.2;
            let _style_token6 = ui.push_style_var(imgui::StyleVar::FramePadding([
                10.0 * padding_multiplier,
                4.0 * padding_multiplier,
            ]));
            
            let _style_token7 = ui.push_style_var(imgui::StyleVar::ButtonTextAlign([0.5, 0.5])); // Center text
            let _style_token8 = ui.push_style_var(imgui::StyleVar::ItemSpacing([8.0, 0.0])); // Spacing between buttons
            
            // Check for hover state before rendering
            let was_hovered = matches!(button.state, ButtonState::Hovered);
            
            // Create button
            let clicked = ui.button_with_size(&button_label, button_size);
            
            // Update button state based on interaction
            let is_hovered = ui.is_item_hovered();
            if is_hovered && button.is_enabled {
                if button.state != ButtonState::Hovered {
                    debug!("Button '{}' entered hover state", button.id);
                }
                button.state = ButtonState::Hovered;
            } else if button.is_enabled && !was_hovered {
                if button.state != ButtonState::Normal {
                    debug!("Button '{}' returned to normal state", button.id);
                }
                button.state = ButtonState::Normal;
            }
            
            // Handle button click
            if clicked && button.is_enabled {
                debug!("Button '{}' clicked! State before: {:?}", button.id, button.state);
                button.state = ButtonState::Active;
                button.click_animation = 1.0;
                button.last_interaction = Some(Instant::now());
                
                // Execute action
                if let Some(ref action) = button.action {
                    action();
                }
                
                // Visual feedback - log the interaction
                debug!("Button '{}' clicked and action executed!", button.id);
            }
            
            // Debug hover state
            if is_hovered {
                debug!("Button '{}' is currently hovered", button.id);
            }
            
            // Pop style vars and colors
            _style_token8.pop();
            _style_token7.pop();
            _style_token6.pop();
            _style_token5.pop();
            _style_token4.pop();
            _style_token3.pop();
            _style_token2.pop();
            _style_token.pop();
            
            // Enhanced tooltip with delay and styling
            if ui.is_item_hovered() {
                // Add a small delay before showing tooltip
                if button.last_interaction.map_or(true, |t| t.elapsed().as_secs_f32() > 0.5) {
                    Self::render_enhanced_tooltip(ui, button);
                }
            }
        }
    }
    
    /// Calculate button color with animations
    fn calculate_button_color(button: &ToolbarButton) -> [f32; 4] {
        let base_color = if !button.is_enabled {
            button.color_theme.disabled
        } else if button.is_active {
            button.color_theme.active
        } else {
            button.color_theme.normal
        };
        
        // Apply hover animation
        let hover_color = Self::interpolate_color(
            base_color,
            button.color_theme.hovered,
            button.hover_progress * 0.3, // Subtle hover effect even in normal state
        );
        
        hover_color
    }
    
    /// Calculate hover color
    fn calculate_hover_color(button: &ToolbarButton) -> [f32; 4] {
        if !button.is_enabled {
            button.color_theme.disabled
        } else {
            button.color_theme.hovered
        }
    }
    
    /// Calculate active color
    fn calculate_active_color(button: &ToolbarButton) -> [f32; 4] {
        if !button.is_enabled {
            button.color_theme.disabled
        } else {
            button.color_theme.active
        }
    }
    
    /// Interpolate between two colors
    fn interpolate_color(color1: [f32; 4], color2: [f32; 4], t: f32) -> [f32; 4] {
        [
            color1[0] + (color2[0] - color1[0]) * t,
            color1[1] + (color2[1] - color1[1]) * t,
            color1[2] + (color2[2] - color1[2]) * t,
            color1[3] + (color2[3] - color1[3]) * t,
        ]
    }
    
    /// Render enhanced tooltip with better styling
    fn render_enhanced_tooltip(ui: &Ui, button: &ToolbarButton) {
        ui.tooltip(|| {
            // Add tooltip header with button name
            ui.text_colored([0.8, 0.8, 1.0, 1.0], &button.icon);
            ui.separator();
            
            // Main tooltip text
            ui.text(button.tooltip);
            
            // Add keyboard shortcut hint if available
            if button.id.contains("sphere") {
                ui.text_disabled("Shortcut: Ctrl+N");
            } else if button.id.contains("box") {
                ui.text_disabled("Shortcut: Ctrl+B");
            } else if button.id.contains("hot_reload") {
                ui.text_disabled("Shortcut: F2");
            } else if button.id.contains("reload") {
                ui.text_disabled("Shortcut: F3");
            }
            
            // Show status
            if !button.is_enabled {
                ui.text_colored([0.8, 0.3, 0.3, 1.0], "Disabled");
            } else if button.is_active {
                ui.text_colored([0.3, 0.8, 0.3, 1.0], "Active");
            }
        });
    }
    
    /// Render a single toolbar button
    #[allow(dead_code)]
    fn render_button(&mut self, ui: &Ui, button: &ToolbarButton) {
        let _is_pressed = button.is_active || button.last_interaction.is_some();
        
        // Use a simple button for now - in full implementation you'd use custom styling
        let button_id = format!("##{}", button.id);
        let clicked = ui.button(&button_id);
        
        if clicked && button.is_enabled {
            // Handle button click
            if let Some(ref action) = button.action {
                action();
            }
        }
        
        // Draw icon on top of button
        let cursor_pos = ui.cursor_pos();
        ui.set_cursor_pos([cursor_pos[0] + 8.0, cursor_pos[1] + 8.0]);
        ui.text(button.icon);
        
        // Reset cursor position for next element
        ui.set_cursor_pos([cursor_pos[0] + 30.0, cursor_pos[1]]);
    }
    
    /// Render shortcuts and tooltips
    fn render_shortcuts(&mut self, ui: &Ui) {
        // Handle keyboard shortcuts
        if ui.is_key_pressed(Key::N) && ui.is_key_down(Key::LeftCtrl) {
            if let Some(action) = self.groups[0].buttons[0].action.as_ref() {
                action();
                self.groups[0].buttons[0].last_interaction = Some(Instant::now());
            }
        }
        
        if ui.is_key_pressed(Key::S) && ui.is_key_down(Key::LeftCtrl) {
            if let Some(action) = self.groups[0].buttons[2].action.as_ref() {
                action();
                self.groups[0].buttons[2].last_interaction = Some(Instant::now());
            }
        }
        
        if ui.is_key_pressed(Key::A) && ui.is_key_down(Key::LeftCtrl) {
            if let Some(action) = self.groups[2].buttons[0].action.as_ref() {
                action();
                self.groups[2].buttons[0].last_interaction = Some(Instant::now());
            }
        }
    }
    
    /// Toggle toolbar visibility
    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
        info!("Toolbar visibility: {}", self.is_visible);
    }
    
    /// Set toolbar position
    #[allow(dead_code)]
    pub fn set_position(&mut self, position: ToolbarPosition) {
        self.position = position;
        debug!("Toolbar position set to {:?}", position);
    }
    
    /// Toggle floating mode
    #[allow(dead_code)]
    pub fn toggle_floating(&mut self) {
        self.is_floating = !self.is_floating;
        info!("Toolbar floating: {}", self.is_floating);
    }
    
    /// Get button by ID
    #[allow(dead_code)]
    pub fn get_button(&mut self, id: &str) -> Option<&mut ToolbarButton> {
        for group in &mut self.groups {
            for button in &mut group.buttons {
                if button.id == id {
                    return Some(button);
                }
            }
        }
        None
    }

    /// Get button by ID (mutable reference)
    #[allow(dead_code)]
    pub fn get_button_mut(&mut self, id: &str) -> Option<&mut ToolbarButton> {
        for group in &mut self.groups {
            for button in &mut group.buttons {
                if button.id == id {
                    return Some(button);
                }
            }
        }
        None
    }
    
    /// Set button active state
    #[allow(dead_code)]
    pub fn set_button_active(&mut self, id: &str, active: bool) -> bool {
        if let Some(button) = self.get_button(id) {
            button.is_active = active;
            true
        } else {
            false
        }
    }
    
    /// Set button enabled state
    #[allow(dead_code)]
    pub fn set_button_enabled(&mut self, id: &str, enabled: bool) -> bool {
        if let Some(button) = self.get_button(id) {
            button.is_enabled = enabled;
            if !enabled {
                button.state = ButtonState::Disabled;
            } else {
                button.state = ButtonState::Normal;
            }
            true
        } else {
            false
        }
    }
    
    /// Trigger button click animation programmatically
    #[allow(dead_code)]
    pub fn trigger_button_animation(&mut self, id: &str) -> bool {
        if let Some(button) = self.get_button(id) {
            button.click_animation = 1.0;
            button.last_interaction = Some(Instant::now());
            true
        } else {
            false
        }
    }
    
    /// Get button current state
    #[allow(dead_code)]
    pub fn get_button_state(&self, id: &str) -> Option<ButtonState> {
        for group in &self.groups {
            for button in &group.buttons {
                if button.id == id {
                    return Some(button.state);
                }
            }
        }
        None
    }
    
    /// Add visual feedback for button interactions
    #[allow(dead_code)]
    pub fn add_interaction_feedback(&mut self, id: &str, feedback_type: InteractionFeedback) {
        if let Some(button) = self.get_button(id) {
            match feedback_type {
                InteractionFeedback::Success => {
                    // Flash green briefly
                    button.click_animation = 1.0;
                    debug!("Button '{}' interaction: Success", id);
                }
                InteractionFeedback::Error => {
                    // Flash red briefly
                    button.click_animation = 1.0;
                    debug!("Button '{}' interaction: Error", id);
                }
                InteractionFeedback::Warning => {
                    // Flash yellow briefly
                    button.click_animation = 1.0;
                    debug!("Button '{}' interaction: Warning", id);
                }
            }
        }
    }
    
    /// Create a pulse effect for important buttons
    #[allow(dead_code)]
    pub fn create_pulse_effect(&mut self, id: &str) {
        if let Some(button) = self.get_button(id) {
            button.hover_progress = 0.5; // Start with half hover animation
            debug!("Pulse effect created for button '{}'", id);
        }
    }
}

/// Types of interaction feedback for buttons
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum InteractionFeedback {
    Success,
    Error,
    Warning,
}
