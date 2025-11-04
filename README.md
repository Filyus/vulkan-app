# Vulkan App - SDF Rendering with ECS Architecture

A comprehensive Vulkan application written in Rust featuring Entity Component System (ECS) architecture with Signed Distance Function (SDF) rendering, complete rendering pipeline, and modern Vulkan best practices using the Ash Vulkan bindings.

## Features

- **SDF Rendering**: Signed Distance Function rendering instead of traditional mesh geometry
- **Multiple SDF Shapes**: Support for spheres, boxes, and planes with ray marching
- **ECS Architecture**: Entity Component System for scalable game/application development
- **Dynamic Lighting**: Phong lighting model with multiple lights and shadows
- **Interactive HUD System**: Professional toolbar with hoverable and clickable buttons
- **Enhanced Button Interactions**: Smooth hover effects, visual feedback, and consistent color themes
- **Real-time Updates**: Dynamic aspect ratio handling and window resize support
- **Windowed Fullscreen**: Smooth fullscreen transitions with F11 toggle
- **Proper Camera System**: Advanced camera module with correct aspect ratio and projection matrix handling
- **Complete Vulkan Implementation**: Full Vulkan setup with instance, device, swapchain, and rendering pipeline
- **Modern Error Handling**: Comprehensive error handling with custom `AppError` types
- **Debug Support**: Extensive debugging utilities and validation layer integration
- **Configuration System**: Centralized configuration for window, Vulkan, rendering, and debug settings
- **Runtime Shader Compilation**: Automatic shader compilation on startup with caching and optimization
- **🔥 Shader Hot Reload**: Real-time shader reloading with file system monitoring and immediate pipeline updates
- **Resource Management**: Proper Vulkan resource cleanup and memory management with enhanced shutdown sequencing

## Prerequisites

1. **Rust** - Install from [rustup.rs](https://rustup.rs/)
2. **Vulkan SDK** - Download and install from [LunarG](https://vulkan.lunarg.com/) - Required for Ash Vulkan bindings and runtime
3. **Visual Studio Build Tools** (on Windows) - Required for building native dependencies

## Building and Running

1. Clone the repository:
```bash
git clone https://github.com/Filyus/vulkan-app.git
cd vulkan-app
```

2. Build and run the application (shaders are compiled automatically):
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
- **HUD System** (`src/hud/`): Interactive heads-up display with:
  - Blender-inspired toolbar interface
  - Hoverable and clickable buttons with smooth animations
  - Enhanced visual feedback and consistent color themes
  - ImGui integration with Vulkan backend
  - Real-time mouse input handling and state management
- **Vulkan Renderer** (`src/vulkan/`): Complete Vulkan implementation including:
  - Instance and device management
  - Swapchain creation and management
  - Graphics pipeline setup
  - Rendering framework
- **Error Handling** (`src/error.rs`): Comprehensive error types with `AppError` enum
- **Debug Utilities** (`src/debug.rs`): Validation layers, debug messaging, and logging
- **Configuration** (`src/config.rs`): Centralized settings for all application components
- **Camera System** (`src/camera.rs`): Advanced camera with proper aspect ratio and projection handling

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
│   ├── pipeline.rs     # Graphics pipeline with runtime shader compilation
│   ├── shader_compiler.rs  # Runtime shader compilation and caching system
│   ├── shader_watcher.rs   # Hot reload system with file system monitoring
│   └── renderer.rs     # Main renderer with enhanced cleanup
└── hud/                 # HUD and UI system
│   ├── mod.rs          # HUD system integration and management
│   ├── toolbar.rs      # Interactive toolbar with buttons
│   ├── imgui_vulkan_backend.rs  # ImGui Vulkan rendering backend
│   └── vulkan_renderer.rs      # Simplified ImGui renderer
└── camera.rs           # Camera system with aspect ratio handling
└── shaders/             # GLSL shader sources
    ├── sdf.vert        # SDF vertex shader (fullscreen quad)
    ├── sdf.frag        # SDF fragment shader (ray marching)
    ├── sdf.vert.spv    # Compiled vertex shader
    └── sdf.frag.spv    # Compiled fragment shader
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
- `imgui` - Immediate mode GUI library
- `imgui-winit-support` - Winit integration for ImGui
- `imgui-glow-renderer` - OpenGL renderer for ImGui (fallback)
- `notify` - File system monitoring for hot reload functionality

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

## Runtime Shader Compilation

This application features advanced runtime shader compilation that automatically compiles GLSL shaders to SPIR-V bytecode during application startup, eliminating the need for manual shader compilation steps.

### Features

- **Automatic Compilation**: Shaders are compiled automatically when the application starts
- **Shader Caching**: Compiled shaders are cached in memory for faster subsequent compilations
- **Debug Support**: Debug information is automatically included in debug builds
- **Optimization**: Performance optimization levels are applied in release builds
- **Error Handling**: Comprehensive error reporting for shader compilation failures
- **Preloading**: Common shaders are preloaded during startup for faster initialization

### Shader Configuration

The shader compilation system is configured through `src/config.rs`:

- **Shader Cache**: Enabled by default for faster compilation
- **Debug Info**: Automatically enabled in debug builds
- **Optimization**: Performance optimization in release, zero optimization in debug
- **Preloading**: Common shaders are preloaded during startup

### Shader Files

- **sdf.vert**: Vertex shader for fullscreen quad rendering
- **sdf.frag**: Fragment shader implementing SDF ray marching with proper aspect ratio handling
- **imgui.vert**: ImGui vertex shader for UI rendering
- **imgui.frag**: ImGui fragment shader for UI rendering

The shaders include:
- Proper aspect ratio correction to prevent stretching during window resize
- Push constants for dynamic window data (resolution, time, aspect ratio)
- SDF ray marching implementation with multiple shape support
- Phong lighting model with shadows
- ImGui integration for UI rendering

## 🔥 Shader Hot Reload System

This application features an advanced shader hot reload system that allows real-time shader modification without restarting the application. Perfect for rapid prototyping and development workflow.

### Features

- **Real-time File Monitoring**: Automatic detection of shader file changes using the `notify` crate
- **Immediate Pipeline Updates**: Shaders are recompiled and pipelines are recreated instantly
- **Thread-safe Operations**: Hot reload manager works on a separate thread to prevent UI blocking
- **Command Buffer Synchronization**: Automatic command buffer recreation after pipeline updates
- **HUD Integration**: Hot reload controls are integrated into the toolbar interface
- **Error Recovery**: Graceful handling of compilation errors with fallback to previous working state
- **Performance Optimized**: Debounced file watching prevents excessive recompilation

### Hot Reload Configuration

The hot reload system is configured through `src/vulkan/shader_watcher.rs`:

- **File Watching**: Monitors `src/shaders/` directory for changes
- **Debounce Delay**: 100ms debounce to prevent rapid recompilation
- **Supported Formats**: `.vert`, `.frag`, `.comp`, `.geom`, `.tesc`, `.tese` shader files
- **Callback System**: Event-driven architecture for pipeline updates
- **Thread Safety**: Arc<Mutex<>> based sharing for concurrent access

### Usage

1. **Automatic Mode**: Hot reload is enabled by default in debug builds
2. **Manual Toggle**: Use the HUD toolbar button to enable/disable hot reload
3. **Real-time Editing**: Edit any shader file in `src/shaders/` and see changes immediately
4. **Error Handling**: Compilation errors are logged but won't crash the application

### File Structure

```
src/shaders/
├── sdf.vert        # Vertex shader (editable for hot reload)
├── sdf.frag        # Fragment shader (editable for hot reload)
├── imgui.vert      # ImGui vertex shader
├── imgui.frag      # ImGui fragment shader
└── *.spv          # Compiled SPIR-V binaries (auto-generated)
```

### Implementation Details

- **HotReloadManager**: Main coordinator for shader watching and pipeline updates
- **ShaderWatcher**: File system monitoring with debounced event handling
- **Pipeline Integration**: Direct pipeline recreation with minimal frame drops
- **Resource Management**: Proper cleanup and recreation of Vulkan resources
- **ECS Integration**: Hot reload status is accessible through the ECS world system

## Troubleshooting

If you encounter build errors:

1. **Vulkan SDK**: Ensure the Vulkan SDK is properly installed and the `VULKAN_SDK` environment variable is set (required for Ash Vulkan bindings)
2. **Build Tools**: Make sure you have the Visual Studio Build Tools installed (on Windows)
3. **ShaderC**: The shaderc library is used for runtime shader compilation and is included as a dependency (no manual shader compilation needed)
4. **Clean Build**: Try running `cargo clean` and then `cargo build` to force a clean rebuild

If the app fails to run with validation layer errors, it will automatically fall back to running without validation layers.

## Contributing

This project demonstrates advanced SDF rendering techniques with ECS architecture and modern UI systems. Key areas for contribution:

- Additional SDF primitives (torus, cylinder, complex shapes)
- Advanced lighting models (PBR, global illumination)
- Performance optimizations (LOD systems, culling)
- Animation and transformation systems
- Material systems and texture support
- Cross-platform improvements
- Additional debug and profiling tools
- Enhanced HUD features (more toolbar buttons, context menus, panels)
- Advanced UI interactions (drag-and-drop, keyboard shortcuts, customizable layouts)
- ImGui integration improvements and additional UI components
- **Shader System Enhancements**:
  - Compute shader support
  - Shader variant management
  - Persistent shader cache to disk
  - Shader dependency tracking
  - Advanced hot reload features (conditional compilation, shader parameter tweaking)

## License

This project is open source. See the LICENSE file for details.