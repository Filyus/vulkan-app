# Vulkan App - SDF Rendering with ECS Architecture

A comprehensive Vulkan application written in Rust featuring Entity Component System (ECS) architecture with Signed Distance Function (SDF) rendering, complete rendering pipeline, and modern Vulkan best practices using the Ash Vulkan bindings.

## Repository

https://github.com/Filyus/vulkan-app.git

## Features

- **SDF Rendering**: Signed Distance Function rendering instead of traditional mesh geometry
- **Multiple SDF Shapes**: Support for spheres, boxes, and planes with ray marching
- **ECS Architecture**: Entity Component System for scalable game/application development
- **Dynamic Lighting**: Phong lighting model with multiple lights and shadows
- **Real-time Updates**: Dynamic aspect ratio handling and window resize support
- **Complete Vulkan Implementation**: Full Vulkan setup with instance, device, swapchain, and rendering pipeline
- **Modern Error Handling**: Comprehensive error handling with custom `AppError` types
- **Debug Support**: Extensive debugging utilities and validation layer integration
- **Configuration System**: Centralized configuration for window, Vulkan, rendering, and debug settings
- **Shader Support**: GLSL vertex and fragment shaders with proper compilation pipeline
- **Resource Management**: Proper Vulkan resource cleanup and memory management

## Prerequisites

1. **Rust** - Install from [rustup.rs](https://rustup.rs/)
2. **Vulkan SDK** - Download and install from [LunarG](https://vulkan.lunarg.com/)
3. **Visual Studio Build Tools** (on Windows) - Required for building native dependencies

## Building and Running

1. Clone the repository:
```bash
git clone https://github.com/Filyus/vulkan-app.git
cd vulkan-app
```

2. Build and run the application:
```bash
cargo run
```

3. Run tests:
```bash
cargo test
```

## Architecture

### Core Components

- **Main Application** (`src/main.rs`): Application entry point with event loop and window management
- **ECS System** (`src/ecs/`): Entity Component System with components, systems, and world management
- **Vulkan Renderer** (`src/vulkan/`): Complete Vulkan implementation including:
  - Instance and device management
  - Swapchain creation and management
  - Graphics pipeline setup
  - Rendering framework
- **Error Handling** (`src/error.rs`): Comprehensive error types with `AppError` enum
- **Debug Utilities** (`src/debug.rs`): Validation layers, debug messaging, and logging
- **Configuration** (`src/config.rs`): Centralized settings for all application components

### Project Structure

```
src/
├── main.rs              # Application entry point
├── config.rs            # Configuration constants and settings
├── debug.rs             # Debug utilities and validation
├── error.rs             # Custom error handling (AppError)
├── ecs/                 # Entity Component System
│   ├── mod.rs          # ECS module exports
│   ├── components.rs   # Entity components
│   ├── systems.rs      # ECS systems
│   └── world.rs        # ECS world management
├── vulkan/              # Vulkan rendering components
│   ├── mod.rs          # Vulkan module exports
│   ├── instance.rs     # Vulkan instance management
│   ├── device.rs       # Vulkan device management
│   ├── swapchain.rs    # Swapchain handling
│   ├── pipeline.rs     # Graphics pipeline
│   └── renderer.rs     # Main renderer
└── shaders/             # GLSL shader sources
    ├── sdf.vert        # SDF vertex shader (fullscreen quad)
    └── sdf.frag        # SDF fragment shader (ray marching)
```

## Dependencies

- `ash` - Vulkan bindings for Rust
- `winit` - Window creation and event handling
- `cgmath` - Math utilities for 3D graphics
- `raw-window-handle` - Window handle abstraction
- `bytemuck` - Safe memory casting for vertex data
- `legion` - Entity Component System framework
- `log` - Logging framework
- `fern` - Logging implementation
- `chrono` - Time utilities for logging

## Configuration

The application uses a comprehensive configuration system located in `src/config.rs`:

- **Window Settings**: Default size (800x600), title, minimum dimensions
- **Vulkan Settings**: API version, validation layers, device extensions
- **Rendering Settings**: Clear color, culling, rasterization parameters
- **Debug Settings**: Logging levels, validation, performance monitoring
- **ECS Settings**: Entity limits, system profiling
- **Shader Settings**: Shader paths and entry points
- **Memory Settings**: Buffer alignment and allocation strategies

## Error Handling

The application features robust error handling with the `AppError` enum that covers:
- Vulkan-specific errors (instance, device, swapchain, pipeline creation)
- Window-related errors (creation, event handling)
- ECS errors (world initialization, system execution)
- I/O errors and generic application errors

## Debug Features

- **Validation Layers**: Automatic enablement in debug builds
- **Debug Messenger**: Comprehensive Vulkan validation output
- **Logging**: Structured logging with configurable levels
- **Performance Monitoring**: Frame time tracking and system profiling
- **Memory Tracking**: Debug utilities for memory management

## Building for Release

For optimized release builds:

```bash
cargo build --release
```

## Troubleshooting

If you encounter build errors:

1. **Vulkan SDK**: Ensure the Vulkan SDK is properly installed and the `VULKAN_SDK` environment variable is set
2. **Build Tools**: Make sure you have the Visual Studio Build Tools installed (on Windows)
3. **Clean Build**: Try running `cargo clean` and then `cargo build` to force a clean rebuild

If the app fails to run with validation layer errors, it will automatically fall back to running without validation layers.

## Contributing

This project demonstrates advanced SDF rendering techniques with ECS architecture. Key areas for contribution:

- Additional SDF primitives (torus, cylinder, complex shapes)
- Advanced lighting models (PBR, global illumination)
- Performance optimizations (LOD systems, culling)
- Animation and transformation systems
- Material systems and texture support
- Cross-platform improvements
- Additional debug and profiling tools

## License

This project is open source. See the LICENSE file for details.