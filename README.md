# Vulkan App - Simple Rust + Vulkan Window

This is a minimal Vulkan application written in Rust that demonstrates basic Vulkan initialization and window creation using the Ash Vulkan bindings.

## Repository

https://github.com/Filyus/vulkan-app.git

## Prerequisites

1. **Rust** - Install from [rustup.rs](https://rustup.rs/)
2. **Vulkan SDK** - Download and install from [LunarG](https://vulkan.lunarg.com/)
3. **Visual Studio Build Tools** (on Windows) - Required for building native dependencies

## Building and Running

1. Clone or download this project
2. Open a terminal in the project directory
3. Run the application:

```bash
cargo run
```

## What This App Shows

- Basic Vulkan instance creation
- Window creation using winit
- Validation layer detection and setup (if available)
- Debug messenger setup for validation layers
- Proper resource cleanup on exit
- Basic event loop handling

## Project Structure

- `src/main.rs` - Main application code with Vulkan setup and window management
- `src/ecs/` - Entity Component System implementation
- `src/vulkan/` - Vulkan rendering components
- `src/error.rs` - Custom error handling with AppError
- `src/debug.rs` - Debug utilities and validation
- `src/config.rs` - Application configuration
- `shaders/` - GLSL shader source files
- `Cargo.toml` - Project dependencies and configuration

## Dependencies

- `ash` - Vulkan bindings for Rust
- `winit` - Window creation and event handling
- `cgmath` - Math utilities (included for future use)
- `raw-window-handle` - Window handle abstraction

## Notes

- This is a minimal example focused on simplicity rather than completeness
- The application creates a window and initializes Vulkan with ECS architecture
- Validation layers are automatically enabled if available, otherwise the app runs without them
- This serves as a starting point for more complex Vulkan applications
- Features custom error handling with AppError type

## Troubleshooting

If you encounter build errors:

1. Make sure the Vulkan SDK is properly installed and the VULKAN_SDK environment variable is set
2. Ensure you have the Visual Studio Build Tools installed (on Windows)
3. Try running `cargo clean` and then `cargo build` to force a clean rebuild

If the app fails to run with validation layer errors, it will automatically fall back to running without validation layers.

## Next Steps

To expand this app, you could:

- Add proper swapchain management for actual rendering to the window
- Implement graphics pipeline setup with vertex and fragment shaders
- Add vertex buffers and render a simple triangle
- Implement proper error handling and resource cleanup
- Add texture mapping
- Implement more complex scenes with multiple objects