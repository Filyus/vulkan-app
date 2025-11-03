mod vulkan;
mod ecs;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use vulkan::VulkanRenderer;
use ecs::ECSWorld;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() {
    println!("Vulkan Triangle Demo - ECS");
    println!("This demo renders a colored triangle using Vulkan with ECS architecture.");
    println!("");

    // Initialize event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan Triangle Demo - ECS")
        .with_inner_size(winit::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .expect("Failed to create window");

    // Initialize Vulkan renderer
    let vulkan_renderer = match VulkanRenderer::new(&window) {
        Ok(renderer) => {
            println!("Vulkan initialized successfully!");
            println!("Using device: {}", renderer.device.get_device_name(&renderer.instance.instance));
            renderer
        }
        Err(e) => {
            println!("Vulkan initialization failed: {}", e);
            println!("This might be due to:");
            println!("- Missing Vulkan drivers");
            println!("- Unsupported hardware");
            println!("- Missing Vulkan SDK");
            println!("- Shader compilation issues");
            return;
        }
    };

    // Initialize ECS world
    let mut ecs_world = ECSWorld::new(vulkan_renderer);
    println!("ECS world initialized successfully!");

    // Main loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                // Update ECS systems
                ecs_world.execute();
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if let Err(e) = ecs_world.draw_frame() {
                    println!("Error during draw frame: {}", e);
                }
            }
            _ => (),
        }
    });
}