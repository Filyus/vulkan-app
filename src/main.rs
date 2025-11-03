mod vulkan;
mod ecs;
mod error;
mod config;
mod debug;
mod camera;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use vulkan::VulkanRenderer;
use ecs::ECSWorld;
use error::Result;
use log::{info, error, debug};


fn main() -> Result<()> {
    // Initialize logging first
    debug::init_logging()?;
    
    // Log debug mode configuration
    if debug::VulkanDebugUtils::is_debug_mode_enabled() {
        info!("Debug mode is enabled");
        info!("{}", debug::VulkanDebugUtils::get_debug_config_summary());
    }
    
    info!("Starting Vulkan App - ECS");
    info!("This app renders SDF shapes using Vulkan with ECS architecture.");
    
    // Initialize event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(config::window::TITLE)
        .with_inner_size(winit::dpi::PhysicalSize::new(
            config::window::DEFAULT_WIDTH,
            config::window::DEFAULT_HEIGHT
        ))
        .with_min_inner_size(winit::dpi::PhysicalSize::new(
            config::window::MIN_WIDTH,
            config::window::MIN_HEIGHT
        ))
        .build(&event_loop)
        .map_err(|e| error::AppError::Window(
            error::WindowError::Creation(format!("Failed to create window: {}", e))
        ))?;

    // Initialize Vulkan renderer
    let vulkan_renderer = VulkanRenderer::new(&window)?;
    info!("Vulkan initialized successfully!");
    info!("Using device: {}", vulkan_renderer.device.get_device_name(&vulkan_renderer.instance.instance));

    // Initialize ECS world
    let mut ecs_world = ECSWorld::new(vulkan_renderer)?;
    info!("ECS world initialized successfully!");

    // Main loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Window close requested, exiting");
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                info!("Window resized to: {}x{}", new_size.width, new_size.height);
                // Handle window resize
                if let Err(e) = ecs_world.handle_window_resize(new_size.width, new_size.height) {
                    error!("Error during window resize: {}", e);
                }
            }
            Event::MainEventsCleared => {
                // Update ECS systems
                if let Err(e) = ecs_world.execute() {
                    error!("Error during ECS execution: {}", e);
                }
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if let Err(e) = ecs_world.draw_frame() {
                    error!("Error during draw frame: {}", e);
                }
            }
            Event::Resumed => {
                debug!("Application resumed");
            }
            Event::Suspended => {
                debug!("Application suspended");
            }
            _ => (),
        }
    });
}