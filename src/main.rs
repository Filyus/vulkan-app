mod vulkan;
mod ecs;
mod error;
mod config;
mod debug;
mod camera;
mod hud;

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
    is_shutting_down: bool,
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
        println!("=== APPLICATION STARTED - resumed() method called ===");
        debug!("resumed() method called");
        
        // Initialize logging first
        if let Err(e) = debug::init_logging() {
            eprintln!("Failed to initialize logging: {}", e);
        }
        
        debug!("Logging initialized");
        
        // Log debug mode configuration
        if debug::VulkanDebugUtils::is_debug_mode_enabled() {
            info!("Debug mode is enabled");
            info!("{}", debug::VulkanDebugUtils::get_debug_config_summary());
        }
        
        info!("Starting Vulkan App - ECS");
        info!("This app renders SDF shapes using Vulkan with ECS architecture.");
        
        debug!("About to create window");
        
        let window_size = winit::dpi::PhysicalSize::new(
            config::window::DEFAULT_WIDTH,
            config::window::DEFAULT_HEIGHT
        );
        
        // Calculate centered position on primary monitor
        let centered_position = {
            // Get the primary monitor or fallback to the first available monitor
            let primary_monitor = event_loop.primary_monitor().or_else(|| {
                event_loop.available_monitors().next()
            });
            
            if let Some(monitor) = primary_monitor {
                let monitor_size = monitor.size();
                let monitor_position = monitor.position();
                
                // Calculate centered position
                let x = monitor_position.x + ((monitor_size.width as i32 - config::window::DEFAULT_WIDTH as i32) / 2);
                let y = monitor_position.y + ((monitor_size.height as i32 - config::window::DEFAULT_HEIGHT as i32) / 2);
                
                winit::dpi::PhysicalPosition::new(x, y)
            } else {
                // Fallback to centered position if no monitor info available
                winit::dpi::PhysicalPosition::new(
                    (1920 - config::window::DEFAULT_WIDTH as i32) / 2,
                    (1080 - config::window::DEFAULT_HEIGHT as i32) / 2
                )
            }
        };
        
        let window_attributes = WindowAttributes::default()
            .with_title(config::window::TITLE)
            .with_inner_size(window_size)
            .with_min_inner_size(winit::dpi::PhysicalSize::new(
                config::window::MIN_WIDTH,
                config::window::MIN_HEIGHT
            ))
            .with_position(centered_position);
        
        debug!("About to create window with attributes");
        let window = event_loop.create_window(window_attributes).expect("Failed to create window");
        debug!("Window created successfully");
        self.original_window_size = window.inner_size();
        self.original_window_position = centered_position;
        debug!("Window size and position set");
        
        // Initialize Vulkan renderer
        match VulkanRenderer::new(&window) {
            Ok(renderer) => {
                self.vulkan_renderer = Some(renderer);
                info!("Vulkan initialized successfully!");
                if let Some(ref renderer) = self.vulkan_renderer {
                    info!("Using device: {}", renderer.device.get_device_name(&renderer.instance.instance));
                }
                debug!("Vulkan renderer stored in AppState");
            }
            Err(e) => {
                error!("Failed to initialize Vulkan: {}", e);
                event_loop.exit();
                return;
            }
        }

        // Initialize ECS world
        info!("=== STARTING ECS WORLD INITIALIZATION ===");
        debug!("About to initialize ECS world");
        debug!("Checking if Vulkan renderer is available for ECS world creation");
        debug!("Vulkan renderer is_some: {}", self.vulkan_renderer.is_some());
        
        if let Some(renderer) = self.vulkan_renderer.take() {
            info!("Vulkan renderer taken, proceeding with ECS world creation");
            info!("Creating ECS world with Vulkan renderer");
            debug!("Vulkan renderer taken successfully, creating ECS world");
            debug!("About to call ECSWorld::new(renderer)");
            
            match ECSWorld::new(renderer) {
                Ok(mut ecs_world) => {
                    info!("=== ECS WORLD CREATED SUCCESSFULLY ===");
                    info!("ECS world created successfully, initializing HUD");
                    debug!("About to call init_hud");
                    
                    // Initialize HUD after ECS world is created
                    // The init_hud method will handle getting the device and render pass internally
                    info!("=== STARTING HUD INITIALIZATION ===");
                    debug!("Calling ecs_world.init_hud(&window)...");
                    debug!("Window inner size: {}x{}", window.inner_size().width, window.inner_size().height);
                    
                    match ecs_world.init_hud(&window) {
                        Ok(()) => {
                            info!("=== HUD INITIALIZED SUCCESSFULLY ===");
                            info!("HUD initialized successfully!");
                            debug!("HUD is now available: {:?}", ecs_world.hud.is_some());
                            
                            // Set up hot reload callbacks after HUD is initialized
                            // Note: We'll skip callback setup for now due to borrowing issues
                            // The F2/F3 keyboard shortcuts in main.rs will handle hot reload functionality
                            info!("Hot reload callbacks skipped due to borrowing constraints - using keyboard shortcuts instead");
                        }
                        Err(e) => {
                            error!("=== HUD INITIALIZATION FAILED ===");
                            error!("Failed to initialize HUD: {}, continuing without HUD", e);
                            error!("HUD initialization error details: {:?}", e);
                            debug!("HUD is still None: {:?}", ecs_world.hud.is_some());
                        }
                    }
                    info!("=== HUD INITIALIZATION COMPLETED ===");
                    debug!("Final HUD state: {:?}", ecs_world.hud.is_some());
                    
                    // Initialize hot reload after HUD is set up
                    info!("=== STARTING HOT RELOAD INITIALIZATION ===");
                    debug!("About to initialize hot reload");
                    match ecs_world.init_hot_reload() {
                        Ok(()) => {
                            info!("=== HOT RELOAD INITIALIZED SUCCESSFULLY ===");
                            info!("Hot reload initialized successfully!");
                            debug!("Hot reload is now available: {:?}", ecs_world.is_hot_reload_enabled());
                        }
                        Err(e) => {
                            error!("=== HOT RELOAD INITIALIZATION FAILED ===");
                            error!("Failed to initialize hot reload: {}, continuing without hot reload", e);
                            error!("Hot reload initialization error details: {:?}", e);
                            debug!("Hot reload is still not available: {:?}", ecs_world.is_hot_reload_enabled());
                        }
                    }
                    info!("=== HOT RELOAD INITIALIZATION COMPLETED ===");
                    
                    self.ecs_world = Some(ecs_world);
                    info!("=== ECS WORLD INITIALIZATION COMPLETED ===");
                    info!("ECS world initialized successfully!");
                    debug!("ECS world stored in AppState");
                }
                Err(e) => {
                    error!("=== ECS WORLD CREATION FAILED ===");
                    error!("Failed to initialize ECS world: {}", e);
                    error!("ECS world initialization error details: {:?}", e);
                    event_loop.exit();
                    return;
                }
            }
        } else {
            error!("=== VULKAN RENDERER UNAVAILABLE ===");
            error!("Failed to take Vulkan renderer for ECS world creation");
            error!("Vulkan renderer was None when trying to create ECS world");
        }
        info!("=== ECS WORLD INITIALIZATION FINISHED ===");
        
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        // Handle mouse events directly for ImGui
        if let Some(ref mut ecs_world) = self.ecs_world {
            if let Some(ref mut hud) = ecs_world.hud {
                match &event {
                    WindowEvent::CursorMoved { position, .. } => {
                        // Directly update ImGui mouse position
                        let io = hud.context_mut();
                        io.mouse_pos = [position.x as f32, position.y as f32];
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        // Directly update ImGui mouse button state
                        let io = hud.context_mut();
                        match button {
                            winit::event::MouseButton::Left => {
                                io.mouse_down[0] = *state == winit::event::ElementState::Pressed;
                            }
                            winit::event::MouseButton::Right => {
                                io.mouse_down[1] = *state == winit::event::ElementState::Pressed;
                            }
                            winit::event::MouseButton::Middle => {
                                io.mouse_down[2] = *state == winit::event::ElementState::Pressed;
                            }
                            _ => {}
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        // Directly update ImGui mouse wheel
                        let io = hud.context_mut();
                        match delta {
                            winit::event::MouseScrollDelta::LineDelta(_, y) => {
                                io.mouse_wheel = *y;
                            },
                            winit::event::MouseScrollDelta::PixelDelta(y) => {
                                io.mouse_wheel = (y.y as f32) / 16.0; // Convert pixels to lines
                            },
                        }
                    }
                    _ => {}
                }
            }
        }
        
        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested, initiating graceful shutdown");
                
                // Set shutdown flag to stop rendering
                self.is_shutting_down = true;
                
                // Wait for current frame to complete before cleanup
                if let Some(ref mut ecs_world) = self.ecs_world {
                    info!("Waiting for current frame to complete before cleanup");
                    if let Err(e) = ecs_world.wait_for_gpu_idle() {
                        error!("Failed to wait for GPU idle during shutdown: {}", e);
                    }
                    
                    info!("GPU idle confirmed, cleaning up HUD system");
                    ecs_world.cleanup_hud();

                    // Clean up hot reload manager to break reference cycles
                    info!("Cleaning up hot reload manager");
                    ecs_world.cleanup_hot_reload();
                }
                  
                info!("Graceful shutdown completed, exiting");
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
            WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                    state: winit::event::ElementState::Pressed,
                    logical_key: Key::Named(NamedKey::F1),
                    ..
                },
                ..
            } => {
                // Toggle HUD visibility on F1 press
                info!("F1 pressed - toggling HUD visibility");
                if let Some(ref mut ecs_world) = self.ecs_world {
                    ecs_world.toggle_hud();
                }
            }
            WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                    state: winit::event::ElementState::Pressed,
                    logical_key: Key::Named(NamedKey::F2),
                    ..
                },
                ..
            } => {
                // Toggle hot reload on F2 press
                info!("F2 pressed - toggling hot reload");
                if let Some(ref mut ecs_world) = self.ecs_world {
                    let current_state = ecs_world.is_hot_reload_enabled();
                    match ecs_world.set_hot_reload_enabled(!current_state) {
                        Ok(()) => {
                            info!("Hot reload toggled to: {}", !current_state);
                        }
                        Err(e) => {
                            error!("Failed to toggle hot reload: {}", e);
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                    state: winit::event::ElementState::Pressed,
                    logical_key: Key::Named(NamedKey::F3),
                    ..
                },
                ..
            } => {
                // Manual shader reload on F3 press
                info!("F3 pressed - manual shader reload");
                if let Some(ref ecs_world) = self.ecs_world {
                    // Reload the main SDF shaders
                    let shaders_to_reload = [
                        "shaders/sdf.vert",
                        "shaders/sdf.frag",
                    ];
                    
                    for shader_path in &shaders_to_reload {
                        match ecs_world.reload_shader(shader_path) {
                            Ok(()) => {
                                info!("Manual reload successful for: {}", shader_path);
                            }
                            Err(e) => {
                                error!("Manual reload failed for {}: {}", shader_path, e);
                            }
                        }
                    }
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
                        if let Err(e) = ecs_world.handle_window_resize(new_size.width, new_size.height, self.window.as_ref().unwrap()) {
                            error!("Error during window resize: {}", e);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // Skip rendering during shutdown
                if self.is_shutting_down {
                    return;
                }
                
                if let Some(ref mut ecs_world) = self.ecs_world {
                    // Draw the main 3D scene first
                    if let Err(e) = ecs_world.draw_frame() {
                        error!("Error during draw frame: {}", e);
                    }
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Skip ECS updates and rendering during shutdown
        if self.is_shutting_down {
            return;
        }
        
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
            if let Err(e) = ecs_world.execute(self.window.as_ref().unwrap(), 0.016) {
                error!("Error during ECS execution: {}", e);
            }
        }
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    println!("=== MAIN FUNCTION STARTED ===");
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
        is_shutting_down: false,
    };
    
    let _ = event_loop.run_app(&mut app);
    Ok(())
}
