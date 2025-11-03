use ash::vk;
use ash::{Device, Instance};
use crate::vulkan::{VulkanInstance, VulkanDevice, VulkanSwapchain, VulkanPipeline};
use crate::error::{Result, VulkanError};
use crate::config;
use crate::camera::Camera;
use winit::window::Window;
use log::{debug, info, error};

// Wrapper for surface to handle proper cleanup
struct SurfaceWrapper {
    surface: vk::SurfaceKHR,
    surface_loader: ash::khr::surface::Instance,
}

impl Drop for SurfaceWrapper {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}

pub struct VulkanRenderer {
    // Order matters for cleanup! Rust drops in reverse order of declaration.
    // Things that depend on others must be declared first.
    
    _vertex_buffer: vk::Buffer,
    _vertex_buffer_memory: vk::DeviceMemory,
    _index_buffer: vk::Buffer,
    _index_buffer_memory: vk::DeviceMemory,
    
    // Sync objects (cleaned up before device)
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    
    // Command pool and buffers (cleaned up before device)
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    
    // Framebuffers (cleaned up before pipeline and swapchain)
    framebuffers: Vec<vk::Framebuffer>,
    
    // Pipeline (cleaned up before device)
    pub pipeline: VulkanPipeline,
    
    // Swapchain (cleaned up before surface and device)
    pub swapchain: VulkanSwapchain,
    
    // Surface (cleaned up before instance, but after swapchain)
    surface: SurfaceWrapper,
    
    // Device (cleaned up before instance)
    pub device: VulkanDevice,
    
    // Instance (cleaned up last)
    pub instance: VulkanInstance,
    
    // Camera for proper projection handling
    pub camera: Camera,
    
    // Runtime state
    current_frame: usize,
    
    // For dynamic push constant updates
    time: f32,
    
    // HUD reference for rendering
    hud_reference: Option<*mut crate::hud::HUD>,
}

impl VulkanRenderer {
    /// Create a new Vulkan renderer
    ///
    /// # Arguments
    /// * `window` - The window to render to
    ///
    /// # Returns
    /// A new VulkanRenderer instance
    ///
    /// # Errors
    /// Returns an error if renderer initialization fails
    pub fn new(window: &Window) -> Result<Self> {
        info!("Initializing Vulkan renderer");
        
        let instance = VulkanInstance::new()
            .map_err(|e| VulkanError::InstanceCreation(format!("Failed to create Vulkan instance: {}", e)))?;
        
        let surface = Self::create_surface(&instance.entry, &instance.instance, window)?;
        let surface_loader = ash::khr::surface::Instance::new(&instance.entry, &instance.instance);
        
        let device = VulkanDevice::new(&instance.instance, &instance.entry, surface)
            .map_err(|e| VulkanError::DeviceCreation(format!("Failed to create Vulkan device: {}", e)))?;
        
        let swapchain = VulkanSwapchain::new(&instance.instance, &instance.entry, &device, surface, window)
            .map_err(|e| VulkanError::SwapchainCreation(format!("Failed to create swapchain: {}", e)))?;
        
        let pipeline = VulkanPipeline::new(&device.device, swapchain.swapchain_image_format)
            .map_err(|e| VulkanError::PipelineCreation(format!("Failed to create pipeline: {}", e)))?;
        
        let framebuffers = Self::create_framebuffers(
            &device.device,
            pipeline.render_pass,
            &swapchain.swapchain_image_views,
            swapchain.swapchain_extent
        )?;
        
        let command_pool = Self::create_command_pool(&device.device, &device.queue_families)?;
        let command_buffers = Self::create_command_buffers(
            &device.device,
            command_pool,
            pipeline.graphics_pipeline,
            pipeline.pipeline_layout,
            pipeline.render_pass,
            &framebuffers,
            swapchain.swapchain_extent
        )?;
        
        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) =
            Self::create_sync_objects(&device.device)?;
        
        
        // Temporarily disable vertex buffer creation to focus on ECS integration
        let vertex_buffer = vk::Buffer::null();
        let vertex_buffer_memory = vk::DeviceMemory::null();
        let index_buffer = vk::Buffer::null();
        let index_buffer_memory = vk::DeviceMemory::null();
        
        // Create camera with proper aspect ratio
        let aspect_ratio = swapchain.swapchain_extent.width as f32 / swapchain.swapchain_extent.height as f32;
        let camera = Camera::with_params(
            cgmath::Point3::new(0.0, 0.0, 2.0),  // position
            cgmath::Point3::new(0.0, 0.0, 0.0),  // target
            cgmath::Vector3::new(0.0, 1.0, 0.0), // up
            cgmath::Deg(45.0).into(),              // fov
            0.1,                                 // near
            100.0,                               // far
            aspect_ratio,                         // aspect ratio
        );
        
        info!("Vulkan renderer initialized successfully");
        
        Ok(Self {
            _vertex_buffer: vertex_buffer,
            _vertex_buffer_memory: vertex_buffer_memory,
            _index_buffer: index_buffer,
            _index_buffer_memory: index_buffer_memory,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            command_pool,
            command_buffers,
            framebuffers,
            pipeline,
            swapchain,
            surface: SurfaceWrapper { surface, surface_loader },
            device,
            instance,
            camera,
            current_frame: 0,
            time: 0.0,
            hud_reference: None,
        })
    }
    
    /// Create a Vulkan surface for the given window
    ///
    /// # Arguments
    /// * `entry` - The Vulkan entry point
    /// * `instance` - The Vulkan instance
    /// * `window` - The window to create a surface for
    ///
    /// # Returns
    /// The created Vulkan surface
    ///
    /// # Errors
    /// Returns an error if surface creation fails
    fn create_surface(
        entry: &ash::Entry,
        instance: &Instance,
        window: &Window
    ) -> Result<vk::SurfaceKHR> {
        use raw_window_handle::HasWindowHandle;
        
        debug!("Creating Vulkan surface");
        
        let handle = window.window_handle()
            .map_err(|e| VulkanError::SurfaceCreation(format!("Failed to get window handle: {:?}", e)))?;
        
        match handle.as_raw() {
            raw_window_handle::RawWindowHandle::Win32(handle) => {
                let win32_create_info = vk::Win32SurfaceCreateInfoKHR::default()
                    .hinstance(handle.hinstance.map(|h| h.get()).unwrap_or(0))
                    .hwnd(handle.hwnd.get());
                
                let surface_loader = ash::khr::win32_surface::Instance::new(entry, instance);
                let surface = unsafe {
                    surface_loader.create_win32_surface(&win32_create_info, None)
                        .map_err(|e| VulkanError::SurfaceCreation(format!("Failed to create Win32 surface: {:?}", e)))?
                };
                
                debug!("Vulkan surface created successfully");
                Ok(surface)
            }
            _ => Err(VulkanError::SurfaceCreation("Unsupported window handle type".to_string()).into()),
        }
    }
    
    /// Create framebuffers for the swapchain images
    ///
    /// # Arguments
    /// * `device` - The Vulkan device
    /// * `render_pass` - The render pass
    /// * `image_views` - The swapchain image views
    /// * `extent` - The extent of the framebuffers
    ///
    /// # Returns
    /// A vector of created framebuffers
    ///
    /// # Errors
    /// Returns an error if framebuffer creation fails
    fn create_framebuffers(
        device: &Device,
        render_pass: vk::RenderPass,
        image_views: &[vk::ImageView],
        extent: vk::Extent2D
    ) -> Result<Vec<vk::Framebuffer>> {
        debug!("Creating {} framebuffers", image_views.len());
        
        let mut framebuffers = vec![];
        
        for (i, &image_view) in image_views.iter().enumerate() {
            let attachments = [image_view];
            
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            
            let framebuffer = unsafe {
                device.create_framebuffer(&framebuffer_info, None)
                    .map_err(|e| VulkanError::PipelineCreation(format!("Failed to create framebuffer {}: {:?}", i, e)))?
            };
            framebuffers.push(framebuffer);
        }
        
        debug!("Created {} framebuffers successfully", framebuffers.len());
        Ok(framebuffers)
    }
    
    /// Create a command pool for command buffer allocation
    ///
    /// # Arguments
    /// * `device` - The Vulkan device
    /// * `indices` - The queue family indices
    ///
    /// # Returns
    /// The created command pool
    ///
    /// # Errors
    /// Returns an error if command pool creation fails
    fn create_command_pool(
        device: &Device,
        indices: &crate::vulkan::device::QueueFamilyIndices
    ) -> Result<vk::CommandPool> {
        debug!("Creating command pool for graphics queue family: {}",
               indices.graphics_family.unwrap());
        
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(indices.graphics_family.unwrap());
        
        let command_pool = unsafe {
            device.create_command_pool(&pool_info, None)
                .map_err(|e| VulkanError::CommandBuffer(format!("Failed to create command pool: {:?}", e)))?
        };
        
        debug!("Command pool created successfully");
        Ok(command_pool)
    }
    
    /// Create command buffers for rendering
    ///
    /// # Arguments
    /// * `device` - The Vulkan device
    /// * `command_pool` - The command pool to allocate from
    /// * `graphics_pipeline` - The graphics pipeline to bind
    /// * `render_pass` - The render pass to use
    /// * `framebuffers` - The framebuffers to render to
    /// * `extent` - The render extent
    ///
    /// # Returns
    /// A vector of created command buffers
    ///
    /// # Errors
    /// Returns an error if command buffer creation or recording fails
    fn create_command_buffers(
        device: &Device,
        command_pool: vk::CommandPool,
        graphics_pipeline: vk::Pipeline,
        pipeline_layout: vk::PipelineLayout,
        render_pass: vk::RenderPass,
        framebuffers: &[vk::Framebuffer],
        extent: vk::Extent2D,
    ) -> Result<Vec<vk::CommandBuffer>> {
        debug!("Creating {} command buffers", framebuffers.len());
        
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(framebuffers.len() as u32);
        
        let command_buffers = unsafe {
            device.allocate_command_buffers(&alloc_info)
                .map_err(|e| VulkanError::CommandBuffer(format!("Failed to allocate command buffers: {:?}", e)))?
        };
        
        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            debug!("Recording command buffer {}", i);
            
            let begin_info = vk::CommandBufferBeginInfo::default();
            unsafe {
                device.begin_command_buffer(command_buffer, &begin_info)
                    .map_err(|e| VulkanError::CommandBuffer(format!("Failed to begin command buffer {}: {:?}", i, e)))?;
            }
            
            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(render_pass)
                .framebuffer(framebuffers[i])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent,
                })
                .clear_values(&[vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: config::rendering::CLEAR_COLOR,
                    },
                }]);
            
            unsafe {
                device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
                device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline);
                
                // Set dynamic viewport and scissor
                let viewport = vk::Viewport {
                    x: 0.0,
                    y: 0.0,
                    width: extent.width as f32,
                    height: extent.height as f32,
                    min_depth: 0.0,
                    max_depth: 1.0,
                };
                device.cmd_set_viewport(command_buffer, 0, &[viewport]);
                
                let scissor = vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent,
                };
                device.cmd_set_scissor(command_buffer, 0, &[scissor]);
                
                // Push window data as push constants
                let aspect_ratio = extent.width as f32 / extent.height as f32;
                let push_constants = [
                    extent.width as f32,      // uResolution.x
                    extent.height as f32,     // uResolution.y
                    0.0 as f32,               // uTime (placeholder)
                    aspect_ratio,             // uAspectRatio
                ];
                device.cmd_push_constants(
                    command_buffer,
                    pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    bytemuck::bytes_of(&push_constants)
                );
                
                device.cmd_draw(command_buffer, 6, 1, 0, 0); // Draw 6 vertices for fullscreen quad
                device.cmd_end_render_pass(command_buffer);
                device.end_command_buffer(command_buffer)
                    .map_err(|e| VulkanError::CommandBuffer(format!("Failed to end command buffer {}: {:?}", i, e)))?;
            }
        }
        
        debug!("Command buffers created and recorded successfully");
        Ok(command_buffers)
    }
    
    /// Create synchronization objects for frame rendering
    ///
    /// # Arguments
    /// * `device` - The Vulkan device
    ///
    /// # Returns
    /// A tuple of (image_available_semaphores, render_finished_semaphores, in_flight_fences)
    ///
    /// # Errors
    /// Returns an error if sync object creation fails
    fn create_sync_objects(device: &Device) -> Result<(Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>)> {
        debug!("Creating synchronization objects for {} frames in flight", config::vulkan::MAX_FRAMES_IN_FLIGHT);
        
        let mut image_available_semaphores = vec![];
        let mut render_finished_semaphores = vec![];
        let mut in_flight_fences = vec![];
        
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default()
            .flags(vk::FenceCreateFlags::SIGNALED);
        
        for i in 0..config::vulkan::MAX_FRAMES_IN_FLIGHT {
            let image_available_semaphore = unsafe {
                device.create_semaphore(&semaphore_info, None)
                    .map_err(|e| VulkanError::Rendering(format!("Failed to create image available semaphore {}: {:?}", i, e)))?
            };
            let render_finished_semaphore = unsafe {
                device.create_semaphore(&semaphore_info, None)
                    .map_err(|e| VulkanError::Rendering(format!("Failed to create render finished semaphore {}: {:?}", i, e)))?
            };
            let in_flight_fence = unsafe {
                device.create_fence(&fence_info, None)
                    .map_err(|e| VulkanError::Rendering(format!("Failed to create in-flight fence {}: {:?}", i, e)))?
            };
            
            image_available_semaphores.push(image_available_semaphore);
            render_finished_semaphores.push(render_finished_semaphore);
            in_flight_fences.push(in_flight_fence);
        }
        
        debug!("Synchronization objects created successfully");
        Ok((image_available_semaphores, render_finished_semaphores, in_flight_fences))
    }
    
    /// Draw a single frame with HUD
    ///
    /// # Arguments
    /// * `hud` - The HUD to render
    ///
    /// # Returns
    /// Ok(()) if the frame was drawn successfully
    /// Err if drawing failed
    ///
    /// # Errors
    /// Returns an error if any part of the drawing process fails
    pub fn draw_frame_with_hud(&mut self, hud: &mut crate::hud::HUD) -> Result<()> {
        debug!("Drawing frame {} with HUD", self.current_frame);
        
        // Update time for animation
        self.time += 0.016; // Approximate 60 FPS
        
        unsafe {
            // Wait for the previous frame to finish with timeout to prevent hanging
            const FENCE_TIMEOUT_NS: u64 = 1_000_000_000; // 1 second timeout
            
            match self.device.device.wait_for_fences(&[self.in_flight_fences[self.current_frame]], true, FENCE_TIMEOUT_NS) {
                Ok(_) => {
                    debug!("Fence wait completed successfully");
                }
                Err(e) => {
                    error!("Fence wait timed out or failed: {:?}. This may indicate a GPU hang.", e);
                    return Err(VulkanError::Rendering(format!("Fence wait failed: {:?}", e)).into());
                }
            }
            
            // Acquire an image from the swapchain
            let (image_index, _) = self.swapchain.swapchain_loader.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null()
            ).map_err(|e| VulkanError::Rendering(format!("Failed to acquire next image: {:?}", e)))?;
            
            // Update push constants with camera matrices
            let extent = self.swapchain.swapchain_extent;
            let push_constants = [
                extent.width as f32,      // uResolution.x
                extent.height as f32,     // uResolution.y
                self.time,                // uTime
                self.camera.aspect_ratio,    // uAspectRatio (from camera)
            ];
            
            // Record command buffer with updated push constants
            let command_buffer = self.command_buffers[image_index as usize];
            
            // Reset and rerecord command buffer
            self.device.device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|e| VulkanError::CommandBuffer(format!("Failed to reset command buffer: {:?}", e)))?;
            
            let begin_info = vk::CommandBufferBeginInfo::default();
            self.device.device.begin_command_buffer(command_buffer, &begin_info)
                .map_err(|e| VulkanError::CommandBuffer(format!("Failed to begin command buffer: {:?}", e)))?;
            
            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.pipeline.render_pass)
                .framebuffer(self.framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent,
                })
                .clear_values(&[vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: config::rendering::CLEAR_COLOR,
                    },
                }]);
            
            self.device.device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            self.device.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.graphics_pipeline);
            
            // Set dynamic viewport and scissor
            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            self.device.device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            
            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            };
            self.device.device.cmd_set_scissor(command_buffer, 0, &[scissor]);
            
            // Push updated constants to both vertex and fragment shaders
            self.device.device.cmd_push_constants(
                command_buffer,
                self.pipeline.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&push_constants)
            );
            
            self.device.device.cmd_draw(command_buffer, 6, 1, 0, 0); // Draw 6 vertices for fullscreen quad
            
            // Render HUD
            debug!("Rendering HUD directly");
            let hud_extent = vk::Extent2D {
                width: extent.width,
                height: extent.height,
            };
            
            // Render ImGui HUD
            if let Err(e) = hud.render(command_buffer, hud_extent) {
                error!("Failed to render HUD: {}", e);
            } else {
                debug!("HUD rendered successfully");
            }
            
            self.device.device.cmd_end_render_pass(command_buffer);
            self.device.device.end_command_buffer(command_buffer)
                .map_err(|e| VulkanError::CommandBuffer(format!("Failed to end command buffer: {:?}", e)))?;
            
            // Reset the fence for this frame
            self.device.device.reset_fences(&[self.in_flight_fences[self.current_frame]])
                .map_err(|e| VulkanError::Rendering(format!("Failed to reset fences: {:?}", e)))?;
            
            // Set up the submission info
            let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
            let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            
            let command_buffers = [command_buffer];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);
            
            // Submit the command buffer
            self.device.device.queue_submit(
                self.device.graphics_queue,
                &[submit_info],
                self.in_flight_fences[self.current_frame]
            ).map_err(|e| VulkanError::Rendering(format!("Failed to submit command buffer: {:?}", e)))?;
            
            // Present the image
            let swapchains = [self.swapchain.swapchain];
            let image_indices = [image_index];
            
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);
            
            self.swapchain.swapchain_loader.queue_present(self.device.present_queue, &present_info)
                .map_err(|e| VulkanError::Rendering(format!("Failed to present image: {:?}", e)))?;
            
            // Advance to the next frame
            self.current_frame = (self.current_frame + 1) % config::vulkan::MAX_FRAMES_IN_FLIGHT;
        }
        
        debug!("Frame {} with HUD completed successfully", self.current_frame);
        Ok(())
    }

    /// Draw a single frame
    ///
    /// # Returns
    /// Ok(()) if the frame was drawn successfully
    /// Err if drawing failed
    ///
    /// # Errors
    /// Returns an error if any part of the drawing process fails
    pub fn draw_frame(&mut self) -> Result<()> {
        debug!("Drawing frame {}", self.current_frame);
        
        // Update time for animation
        self.time += 0.016; // Approximate 60 FPS
        
        unsafe {
            // Wait for the previous frame to finish with timeout to prevent hanging
            const FENCE_TIMEOUT_NS: u64 = 1_000_000_000; // 1 second timeout
            
            match self.device.device.wait_for_fences(&[self.in_flight_fences[self.current_frame]], true, FENCE_TIMEOUT_NS) {
                Ok(_) => {
                    debug!("Fence wait completed successfully");
                }
                Err(e) => {
                    error!("Fence wait timed out or failed: {:?}. This may indicate a GPU hang.", e);
                    return Err(VulkanError::Rendering(format!("Fence wait failed: {:?}", e)).into());
                }
            }
            
            // Acquire an image from the swapchain
            let (image_index, _) = self.swapchain.swapchain_loader.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null()
            ).map_err(|e| VulkanError::Rendering(format!("Failed to acquire next image: {:?}", e)))?;
            
            // Update push constants with camera matrices
            let extent = self.swapchain.swapchain_extent;
            let push_constants = [
                extent.width as f32,      // uResolution.x
                extent.height as f32,     // uResolution.y
                self.time,                // uTime
                self.camera.aspect_ratio,    // uAspectRatio (from camera)
            ];
            
            // Record command buffer with updated push constants
            let command_buffer = self.command_buffers[image_index as usize];
            
            // Reset and rerecord command buffer
            self.device.device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|e| VulkanError::CommandBuffer(format!("Failed to reset command buffer: {:?}", e)))?;
            
            let begin_info = vk::CommandBufferBeginInfo::default();
            self.device.device.begin_command_buffer(command_buffer, &begin_info)
                .map_err(|e| VulkanError::CommandBuffer(format!("Failed to begin command buffer: {:?}", e)))?;
            
            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.pipeline.render_pass)
                .framebuffer(self.framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent,
                })
                .clear_values(&[vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: config::rendering::CLEAR_COLOR,
                    },
                }]);
            
            self.device.device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            self.device.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.graphics_pipeline);
            
            // Set dynamic viewport and scissor
            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            self.device.device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            
            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            };
            self.device.device.cmd_set_scissor(command_buffer, 0, &[scissor]);
            
            // Push updated constants to both vertex and fragment shaders
            self.device.device.cmd_push_constants(
                command_buffer,
                self.pipeline.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&push_constants)
            );
            
            self.device.device.cmd_draw(command_buffer, 6, 1, 0, 0); // Draw 6 vertices for fullscreen quad
            
            // Render HUD if available
            if let Some(hud) = self.get_hud_for_rendering() {
                debug!("HUD found for rendering, calling HUD render method");
                let hud_extent = vk::Extent2D {
                    width: extent.width,
                    height: extent.height,
                };
                
                // Render ImGui HUD
                if let Err(e) = hud.render(command_buffer, hud_extent) {
                    error!("Failed to render HUD: {}", e);
                } else {
                    debug!("HUD rendered successfully");
                }
            } else {
                debug!("No HUD available for rendering");
            }
            
            self.device.device.cmd_end_render_pass(command_buffer);
            self.device.device.end_command_buffer(command_buffer)
                .map_err(|e| VulkanError::CommandBuffer(format!("Failed to end command buffer: {:?}", e)))?;
            
            // Reset the fence for this frame
            self.device.device.reset_fences(&[self.in_flight_fences[self.current_frame]])
                .map_err(|e| VulkanError::Rendering(format!("Failed to reset fences: {:?}", e)))?;
            
            // Set up the submission info
            let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
            let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            
            let command_buffers = [command_buffer];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);
            
            // Submit the command buffer
            self.device.device.queue_submit(
                self.device.graphics_queue,
                &[submit_info],
                self.in_flight_fences[self.current_frame]
            ).map_err(|e| VulkanError::Rendering(format!("Failed to submit command buffer: {:?}", e)))?;
            
            // Present the image
            let swapchains = [self.swapchain.swapchain];
            let image_indices = [image_index];
            
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);
            
            self.swapchain.swapchain_loader.queue_present(self.device.present_queue, &present_info)
                .map_err(|e| VulkanError::Rendering(format!("Failed to present image: {:?}", e)))?;
            
            // Advance to the next frame
            self.current_frame = (self.current_frame + 1) % config::vulkan::MAX_FRAMES_IN_FLIGHT;
        }
        
        debug!("Frame {} completed successfully", self.current_frame);
        Ok(())
    }
    
    
    /// Handle window resize
    ///
    /// # Arguments
    /// * `new_width` - The new window width
    /// * `new_height` - The new window height
    ///
    /// # Returns
    /// * Ok(()) if resize was handled successfully
    /// * Err if resize handling failed
    pub fn handle_resize(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        info!("Handling window resize to {}x{}", new_width, new_height);
        
        // Use safe device wait to prevent hanging
        match self.device.safe_device_wait_idle() {
            Ok(_) => {
                debug!("Device wait idle completed successfully");
            }
            Err(e) => {
                error!("Failed to wait for device idle during resize: {}. Attempting to continue anyway.", e);
                // Continue with swapchain recreation even if device wait fails
                // This prevents the application from hanging during fullscreen transitions
            }
        }
        
        // Recreate swapchain with error handling
        match self.swapchain.recreate(&self.device, &self.instance.instance, &self.instance.entry, self.surface.surface, new_width, new_height) {
            Ok(_) => {
                debug!("Swapchain recreated successfully");
            }
            Err(e) => {
                error!("Failed to recreate swapchain: {}. Attempting partial recovery.", e);
                // Try to continue with existing swapchain if recreation fails
                return Err(VulkanError::SwapchainCreation(format!("Failed to recreate swapchain: {}", e)).into());
            }
        }
        
        // Update camera aspect ratio
        let new_aspect_ratio = new_width as f32 / new_height as f32;
        self.camera.set_aspect_ratio(new_aspect_ratio);
        
        // Recreate framebuffers with error handling
        if let Err(e) = self.recreate_framebuffers() {
            error!("Failed to recreate framebuffers: {}. Vulkan state may be inconsistent.", e);
            return Err(e);
        }
        
        // Recreate command buffers with error handling
        if let Err(e) = self.recreate_command_buffers() {
            error!("Failed to recreate command buffers: {}. Vulkan state may be inconsistent.", e);
            return Err(e);
        }
        
        info!("Window resize handled successfully");
        Ok(())
    }
    
    /// Recreate framebuffers after resize
    ///
    /// # Returns
    /// * Ok(()) if framebuffers were recreated successfully
    /// * Err if framebuffer recreation failed
    fn recreate_framebuffers(&mut self) -> Result<()> {
        // Clean up old framebuffers
        unsafe {
            for &framebuffer in &self.framebuffers {
                self.device.device.destroy_framebuffer(framebuffer, None);
            }
        }
        
        // Create new framebuffers
        self.framebuffers = Self::create_framebuffers(
            &self.device.device,
            self.pipeline.render_pass,
            &self.swapchain.swapchain_image_views,
            self.swapchain.swapchain_extent
        )?;
        
        Ok(())
    }
    
    /// Recreate command buffers after resize
    ///
    /// # Returns
    /// * Ok(()) if command buffers were recreated successfully
    /// * Err if command buffer recreation failed
    fn recreate_command_buffers(&mut self) -> Result<()> {
        // Free old command buffers
        unsafe {
            self.device.device.free_command_buffers(self.command_pool, &self.command_buffers);
        }
        
        // Create new command buffers
        self.command_buffers = Self::create_command_buffers(
            &self.device.device,
            self.command_pool,
            self.pipeline.graphics_pipeline,
            self.pipeline.pipeline_layout,
            self.pipeline.render_pass,
            &self.framebuffers,
            self.swapchain.swapchain_extent
        )?;
        
        Ok(())
    }
    
    
    /// Get HUD reference for rendering (unsafe - used during render pass)
    fn get_hud_for_rendering(&self) -> Option<&mut crate::hud::HUD> {
        debug!("Getting HUD reference for rendering, current reference: {:?}", self.hud_reference);
        unsafe {
            self.hud_reference.map(|ptr| {
                debug!("Dereferencing HUD pointer: {:?}", ptr);
                &mut *ptr
            })
        }
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            // Wait for device to be idle before cleanup
            let _ = self.device.device.device_wait_idle();
            
            // Clean up sync objects first
            for &fence in &self.in_flight_fences {
                self.device.device.destroy_fence(fence, None);
            }
            for &semaphore in &self.render_finished_semaphores {
                self.device.device.destroy_semaphore(semaphore, None);
            }
            for &semaphore in &self.image_available_semaphores {
                self.device.device.destroy_semaphore(semaphore, None);
            }
            
            // Clean up command pool (this will clean up command buffers)
            self.device.device.destroy_command_pool(self.command_pool, None);
            
            // Clean up framebuffers
            for &framebuffer in &self.framebuffers {
                self.device.device.destroy_framebuffer(framebuffer, None);
            }
            
            // Clean up vertex and index buffers if they exist
            if self._vertex_buffer != vk::Buffer::null() {
                self.device.device.destroy_buffer(self._vertex_buffer, None);
            }
            if self._vertex_buffer_memory != vk::DeviceMemory::null() {
                self.device.device.free_memory(self._vertex_buffer_memory, None);
            }
            if self._index_buffer != vk::Buffer::null() {
                self.device.device.destroy_buffer(self._index_buffer, None);
            }
            if self._index_buffer_memory != vk::DeviceMemory::null() {
                self.device.device.free_memory(self._index_buffer_memory, None);
            }
        }
    }
}