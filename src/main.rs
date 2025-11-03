mod vulkan;
mod ecs;
mod error;
mod config;
mod debug;
mod camera;

use winit::event::WindowEvent;
use winit::event_loop::{EventLoop, ActiveEventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{WindowAttributes, Window};
use winit::application::ApplicationHandler;
use vulkan::VulkanRenderer;
use ecs::ECSWorld;
use error::Result;
use log::{info, error, debug};

struct AppState {
    window: Option<Window>,
    vulkan_renderer: Option<VulkanRenderer>,
    ecs_world: Option<ECSWorld>,
    is_fullscreen: bool,
    fullscreen_pending: bool,
    toggle_fullscreen_flag: bool,
    original_window_size: winit::dpi::PhysicalSize<u32>,
    original_window_position: winit::dpi::PhysicalPosition<i32>,
    original_decorations: bool,
}

impl AppState {
    /// Enter windowed fullscreen mode (borderless window covering the entire screen)
    fn enter_windowed_fullscreen(&mut self, window: &Window) {
        if self.is_fullscreen {
            debug!("Already in fullscreen, ignoring enter request");
            return; // Already in fullscreen
        }
        
        // Store current window state
        self.original_window_size = window.inner_size();
        self.original_window_position = window.outer_position().unwrap_or_else(|_| {
            winit::dpi::PhysicalPosition::new(100, 100)
        });
        self.original_decorations = window.is_decorated();
        
        debug!("Stored window state: {}x{} at ({}, {}), decorations: {}",
               self.original_window_size.width, self.original_window_size.height,
               self.original_window_position.x, self.original_window_position.y,
               self.original_decorations);
        
        // Get the current monitor's dimensions
        let monitor = window.current_monitor();
        let monitor_size = if let Some(ref monitor) = monitor {
            monitor.size()
        } else {
            // Fallback to primary monitor
            winit::dpi::PhysicalSize::new(1920, 1080)
        };
        
        info!("Entering windowed fullscreen: {}x{}", monitor_size.width, monitor_size.height);
        
        // Set the pending flag BEFORE making window changes to prevent race conditions
        self.fullscreen_pending = true;
        self.is_fullscreen = true;
        
        // Remove decorations first
        window.set_decorations(false);
        
        // Position window at top-left of monitor before resizing
        if let Some(ref monitor) = monitor {
            let monitor_pos = monitor.position();
            window.set_outer_position(winit::dpi::PhysicalPosition::new(
                monitor_pos.x + config::windowed_fullscreen::SCREEN_EDGE_OFFSET as i32,
                monitor_pos.y + config::windowed_fullscreen::SCREEN_EDGE_OFFSET as i32
            ));
        }
        
        // Resize to cover entire monitor
        let _ = window.request_inner_size(monitor_size);
        
        debug!("Windowed fullscreen entry initiated");
    }
    
    /// Exit windowed fullscreen mode and restore original window state
    fn exit_windowed_fullscreen(&mut self, window: &Window) {
        if !self.is_fullscreen {
            debug!("Not in fullscreen, ignoring exit request");
            return; // Not in fullscreen
        }
        
        info!("Exiting windowed fullscreen: {}x{} at ({}, {})",
              self.original_window_size.width, self.original_window_size.height,
              self.original_window_position.x, self.original_window_position.y);
        
        // Set the pending flag BEFORE making window changes to prevent race conditions
        self.fullscreen_pending = true;
        self.is_fullscreen = false;
        
        // Restore decorations first
        window.set_decorations(self.original_decorations);
        
        // Restore position first, then size
        window.set_outer_position(self.original_window_position);
        
        // Resize to original size
        let _ = window.request_inner_size(self.original_window_size);
        
        debug!("Windowed fullscreen exit initiated");
    }
    
    /// Toggle windowed fullscreen mode
    fn toggle_windowed_fullscreen(&mut self, window: &Window) {
        debug!("Toggling windowed fullscreen, current state: {}", self.is_fullscreen);
        if self.is_fullscreen {
            self.exit_windowed_fullscreen(window);
        } else {
            self.enter_windowed_fullscreen(window);
        }
    }
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Initialize logging first
        if let Err(e) = debug::init_logging() {
            eprintln!("Failed to initialize logging: {}", e);
        }
        
        // Log debug mode configuration
        if debug::VulkanDebugUtils::is_debug_mode_enabled() {
            info!("Debug mode is enabled");
            info!("{}", debug::VulkanDebugUtils::get_debug_config_summary());
        }
        
        info!("Starting Vulkan App - ECS");
        info!("This app renders SDF shapes using Vulkan with ECS architecture.");
        
        let window_attributes = WindowAttributes::default()
            .with_title(config::window::TITLE)
            .with_inner_size(winit::dpi::PhysicalSize::new(
                config::window::DEFAULT_WIDTH,
                config::window::DEFAULT_HEIGHT
            ))
            .with_min_inner_size(winit::dpi::PhysicalSize::new(
                config::window::MIN_WIDTH,
                config::window::MIN_HEIGHT
            ));
        
        let window = event_loop.create_window(window_attributes).expect("Failed to create window");
        self.original_window_size = window.inner_size();
        self.original_window_position = window.outer_position().unwrap_or_else(|_| {
            winit::dpi::PhysicalPosition::new(100, 100)
        });
        
        // Initialize Vulkan renderer
        match VulkanRenderer::new(&window) {
            Ok(renderer) => {
                self.vulkan_renderer = Some(renderer);
                info!("Vulkan initialized successfully!");
                if let Some(ref renderer) = self.vulkan_renderer {
                    info!("Using device: {}", renderer.device.get_device_name(&renderer.instance.instance));
                }
            }
            Err(e) => {
                error!("Failed to initialize Vulkan: {}", e);
                event_loop.exit();
                return;
            }
        }

        // Initialize ECS world
        if let Some(renderer) = self.vulkan_renderer.take() {
            match ECSWorld::new(renderer) {
                Ok(ecs_world) => {
                    self.ecs_world = Some(ecs_world);
                    info!("ECS world initialized successfully!");
                }
                Err(e) => {
                    error!("Failed to initialize ECS world: {}", e);
                    event_loop.exit();
                    return;
                }
            }
        }
        
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested, exiting");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                    state: winit::event::ElementState::Pressed,
                    logical_key: Key::Named(NamedKey::F11),
                    ..
                },
                ..
            } => {
                // Toggle windowed fullscreen on F11 press
                if config::windowed_fullscreen::ENABLED {
                    info!("F11 pressed - toggling windowed fullscreen");
                    // Use a flag to avoid borrowing issues
                    let should_toggle = self.window.is_some();
                    if should_toggle {
                        // Safe to call toggle_windowed_fullscreen without borrowing window here
                        self.toggle_fullscreen_flag = true;
                    }
                } else {
                    debug!("F11 pressed - windowed fullscreen disabled in config");
                }
            }
            WindowEvent::Resized(new_size) => {
                info!("Window resized to: {}x{} (fullscreen_pending: {})", new_size.width, new_size.height, self.fullscreen_pending);
                
                // If we have a pending fullscreen toggle, handle it instead of normal resize
                if self.fullscreen_pending {
                    info!("Handling pending windowed fullscreen toggle after resize");
                    
                    // First, handle the Vulkan resource recreation
                    if let Some(ref mut ecs_world) = self.ecs_world {
                        if let Some(window) = self.window.as_ref() {
                            // Use a timeout to prevent hanging if Vulkan operations fail
                            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                ecs_world.handle_fullscreen_toggle(window)
                            })) {
                                Ok(result) => {
                                    if let Err(e) = result {
                                        error!("Error during windowed fullscreen toggle: {}", e);
                                    }
                                }
                                Err(_) => {
                                    error!("Panic during fullscreen toggle - Vulkan resource recreation failed");
                                }
                            }
                        }
                    }
                    
                    // Clear the pending flag after handling
                    self.fullscreen_pending = false;
                    info!("Cleared fullscreen_pending flag");
                    
                    // Force a redraw to ensure the display is updated
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                } else {
                    // Handle normal window resize (not during fullscreen toggle)
                    info!("Handling normal window resize");
                    if let Some(ref mut ecs_world) = self.ecs_world {
                        if let Err(e) = ecs_world.handle_window_resize(new_size.width, new_size.height) {
                            error!("Error during window resize: {}", e);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(ref mut ecs_world) = self.ecs_world {
                    if let Err(e) = ecs_world.draw_frame() {
                        error!("Error during draw frame: {}", e);
                    }
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Handle fullscreen toggle flag if set
        if self.toggle_fullscreen_flag {
            self.toggle_fullscreen_flag = false;
            // Take the window to avoid borrowing issues
            if let Some(window) = self.window.take() {
                self.toggle_windowed_fullscreen(&window);
                self.window = Some(window);
            }
        }
        
        // Update ECS systems
        if let Some(ref mut ecs_world) = self.ecs_world {
            if let Err(e) = ecs_world.execute() {
                error!("Error during ECS execution: {}", e);
            }
        }
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = AppState {
        window: None,
        vulkan_renderer: None,
        ecs_world: None,
        is_fullscreen: false,
        fullscreen_pending: false,
        toggle_fullscreen_flag: false,
        original_window_size: winit::dpi::PhysicalSize::new(800, 600),
        original_window_position: winit::dpi::PhysicalPosition::new(100, 100),
        original_decorations: true,
    };
    
    let _ = event_loop.run_app(&mut app);
    Ok(())
}