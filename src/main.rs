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
    original_window_size: winit::dpi::PhysicalSize<u32>,
    original_window_position: winit::dpi::PhysicalPosition<i32>,
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
        let window = self.window.as_ref().unwrap();
        
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
                // F11 fullscreen functionality disabled by default (not yet stable)
                // To enable, uncomment the code below
                /*
                // Toggle fullscreen on F11 press
                info!("F11 pressed - toggling fullscreen");
                
                if !self.is_fullscreen {
                    // Store current window state before entering fullscreen
                    self.original_window_position = window.outer_position().unwrap_or_else(|_| {
                        winit::dpi::PhysicalPosition::new(100, 100)
                    });
                    info!("Storing window state: {}x{} at ({}, {})",
                          self.original_window_size.width, self.original_window_size.height,
                          self.original_window_position.x, self.original_window_position.y);
                }
                
                self.is_fullscreen = !self.is_fullscreen;
                self.fullscreen_pending = true;
                
                if self.is_fullscreen {
                    info!("Entering fullscreen mode");
                    window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(window.current_monitor())));
                } else {
                    info!("Exiting fullscreen mode");
                    window.set_fullscreen(None);
                }
                */
                debug!("F11 pressed - fullscreen functionality disabled");
            }
            WindowEvent::Resized(new_size) => {
                info!("Window resized to: {}x{}", new_size.width, new_size.height);
                
                // Handle window resize
                if let Some(ref mut ecs_world) = self.ecs_world {
                    if let Err(e) = ecs_world.handle_window_resize(new_size.width, new_size.height) {
                        error!("Error during window resize: {}", e);
                    }
                }
                
                // If we have a pending fullscreen toggle, handle it after resize
                if self.fullscreen_pending {
                    info!("Handling pending fullscreen toggle after resize");
                    if let Some(ref mut ecs_world) = self.ecs_world {
                        if let Err(e) = ecs_world.handle_fullscreen_toggle(window) {
                            error!("Error during fullscreen toggle: {}", e);
                        }
                    }
                    
                    // If we're exiting fullscreen, restore window size and position
                    if !self.is_fullscreen {
                        info!("Restoring window size and position after exiting fullscreen");
                        // Note: In newer winit versions, we can't directly set window size
                        // The window will be resized automatically when exiting fullscreen
                        info!("Window size will be restored automatically");
                        
                        window.set_outer_position(winit::dpi::PhysicalPosition::new(
                            self.original_window_position.x,
                            self.original_window_position.y
                        ));
                    }
                    
                    self.fullscreen_pending = false;
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
        original_window_size: winit::dpi::PhysicalSize::new(800, 600),
        original_window_position: winit::dpi::PhysicalPosition::new(100, 100),
    };
    
    let _ = event_loop.run_app(&mut app);
    Ok(())
}