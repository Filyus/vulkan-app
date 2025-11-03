use ash::vk;
use ash::{Device, Instance};
use crate::vulkan::{VulkanInstance, VulkanDevice, VulkanSwapchain, VulkanPipeline};
use crate::ecs::components::Vertex;
use winit::window::Window;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

// Wrapper for surface to handle proper cleanup
struct SurfaceWrapper {
    surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,
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
    
    // ECS-related fields (cleaned up first)
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
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
    #[allow(dead_code)]
    pub pipeline: VulkanPipeline,
    
    // Swapchain (cleaned up before surface and device)
    pub swapchain: VulkanSwapchain,
    
    // Surface (cleaned up before instance, but after swapchain)
    #[allow(dead_code)]
    surface: SurfaceWrapper,
    
    // Device (cleaned up before instance)
    pub device: VulkanDevice,
    
    // Instance (cleaned up last)
    pub instance: VulkanInstance,
    
    // Runtime state
    current_frame: usize,
}

impl VulkanRenderer {
    pub fn new(window: &Window) -> Result<Self, Box<dyn std::error::Error>> {
        let instance = VulkanInstance::new()?;
        let surface = Self::create_surface(&instance.entry, &instance.instance, window)?;
        let surface_loader = ash::extensions::khr::Surface::new(&instance.entry, &instance.instance);
        
        let device = VulkanDevice::new(&instance.instance, &instance.entry, surface)?;
        
        let swapchain = VulkanSwapchain::new(&instance.instance, &instance.entry, &device, surface, window)?;
        
        let pipeline = VulkanPipeline::new(&device.device, swapchain.swapchain_image_format)?;
        
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
            pipeline.render_pass, 
            &framebuffers, 
            swapchain.swapchain_extent
        )?;
        
        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) = 
            Self::create_sync_objects(&device.device)?;
        
        // Create default triangle vertices for initial setup
        let default_vertices = vec![
            Vertex {
                position: cgmath::Vector3::new(0.0, 0.5, 0.0),
                color: cgmath::Vector3::new(1.0, 0.0, 0.0),
            },
            Vertex {
                position: cgmath::Vector3::new(-0.5, -0.5, 0.0),
                color: cgmath::Vector3::new(0.0, 1.0, 0.0),
            },
            Vertex {
                position: cgmath::Vector3::new(0.5, -0.5, 0.0),
                color: cgmath::Vector3::new(0.0, 0.0, 1.0),
            },
        ];
        let default_indices = vec![0, 1, 2];
        
        // Temporarily disable vertex buffer creation to focus on ECS integration
        let vertex_buffer = vk::Buffer::null();
        let vertex_buffer_memory = vk::DeviceMemory::null();
        let index_buffer = vk::Buffer::null();
        let index_buffer_memory = vk::DeviceMemory::null();
        
        Ok(Self {
            vertices: default_vertices,
            indices: default_indices,
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
            current_frame: 0,
        })
    }
    
    fn create_surface(
        entry: &ash::Entry, 
        instance: &Instance, 
        window: &Window
    ) -> Result<vk::SurfaceKHR, Box<dyn std::error::Error>> {
        use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
        
        let handle = window.raw_window_handle();
        match handle {
            RawWindowHandle::Win32(handle) => {
                let win32_create_info = vk::Win32SurfaceCreateInfoKHR::builder()
                    .hinstance(handle.hinstance)
                    .hwnd(handle.hwnd as *const std::os::raw::c_void);
                
                let surface_loader = ash::extensions::khr::Win32Surface::new(entry, instance);
                let surface = unsafe {
                    surface_loader.create_win32_surface(&win32_create_info, None)?
                };
                Ok(surface)
            }
            _ => Err("Unsupported window handle type".into()),
        }
    }
    
    fn create_framebuffers(
        device: &Device, 
        render_pass: vk::RenderPass, 
        image_views: &[vk::ImageView], 
        extent: vk::Extent2D
    ) -> Result<Vec<vk::Framebuffer>, Box<dyn std::error::Error>> {
        let mut framebuffers = vec![];
        
        for &image_view in image_views {
            let attachments = [image_view];
            
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            
            let framebuffer = unsafe { device.create_framebuffer(&framebuffer_info, None)? };
            framebuffers.push(framebuffer);
        }
        
        Ok(framebuffers)
    }
    
    fn create_command_pool(
        device: &Device, 
        indices: &crate::vulkan::device::QueueFamilyIndices
    ) -> Result<vk::CommandPool, Box<dyn std::error::Error>> {
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(indices.graphics_family.unwrap());
        
        let command_pool = unsafe { device.create_command_pool(&pool_info, None)? };
        Ok(command_pool)
    }
    
    fn create_command_buffers(
        device: &Device,
        command_pool: vk::CommandPool,
        graphics_pipeline: vk::Pipeline,
        render_pass: vk::RenderPass,
        framebuffers: &[vk::Framebuffer],
        extent: vk::Extent2D,
    ) -> Result<Vec<vk::CommandBuffer>, Box<dyn std::error::Error>> {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(framebuffers.len() as u32);
        
        let command_buffers = unsafe { device.allocate_command_buffers(&alloc_info)? };
        
        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            let begin_info = vk::CommandBufferBeginInfo::builder();
            unsafe {
                device.begin_command_buffer(command_buffer, &begin_info)?;
            }
            
            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(render_pass)
                .framebuffer(framebuffers[i])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent,
                })
                .clear_values(&[vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                }]);
            
            unsafe {
                device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
                device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline);
                device.cmd_draw(command_buffer, 3, 1, 0, 0);
                device.cmd_end_render_pass(command_buffer);
                device.end_command_buffer(command_buffer)?;
            }
        }
        
        Ok(command_buffers)
    }
    
    fn create_sync_objects(device: &Device) -> Result<(Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>), Box<dyn std::error::Error>> {
        let mut image_available_semaphores = vec![];
        let mut render_finished_semaphores = vec![];
        let mut in_flight_fences = vec![];
        
        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);
        
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let image_available_semaphore = unsafe { device.create_semaphore(&semaphore_info, None)? };
            let render_finished_semaphore = unsafe { device.create_semaphore(&semaphore_info, None)? };
            let in_flight_fence = unsafe { device.create_fence(&fence_info, None)? };
            
            image_available_semaphores.push(image_available_semaphore);
            render_finished_semaphores.push(render_finished_semaphore);
            in_flight_fences.push(in_flight_fence);
        }
        
        Ok((image_available_semaphores, render_finished_semaphores, in_flight_fences))
    }
    
    pub fn draw_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            self.device.device.wait_for_fences(&[self.in_flight_fences[self.current_frame]], true, u64::MAX)?;
            
            let (image_index, _) = self.swapchain.swapchain_loader.acquire_next_image(
                self.swapchain.swapchain, 
                u64::MAX, 
                self.image_available_semaphores[self.current_frame], 
                vk::Fence::null()
            )?;
            
            self.device.device.reset_fences(&[self.in_flight_fences[self.current_frame]])?;
            
            let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
            let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            
            let command_buffers = [self.command_buffers[image_index as usize]];
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores)
                .build();
            
            self.device.device.queue_submit(
                self.device.graphics_queue, 
                &[submit_info], 
                self.in_flight_fences[self.current_frame]
            )?;
            
            let swapchains = [self.swapchain.swapchain];
            let image_indices = [image_index];
            
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);
            
            self.swapchain.swapchain_loader.queue_present(self.device.present_queue, &present_info)?;
            
            self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        }
        
        Ok(())
    }
    
    pub fn update_vertices(&mut self, vertices: &[Vertex]) {
        self.vertices = vertices.to_vec();
        // In a real implementation, we would update the vertex buffer here
        // For now, we'll just store the data
    }
    
    pub fn update_indices(&mut self, indices: &[u32]) {
        self.indices = indices.to_vec();
        // In a real implementation, we would update the index buffer here
        // For now, we'll just store the data
    }
    
    #[allow(dead_code)]
    fn create_vertex_buffer(
        device: &Device,
        vertices: &[Vertex],
    ) -> Result<(vk::Buffer, vk::DeviceMemory), Box<dyn std::error::Error>> {
        let buffer_size = (std::mem::size_of::<Vertex>() * vertices.len()) as vk::DeviceSize;
        
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        
        let buffer = unsafe { device.create_buffer(&buffer_info, None)? };
        
        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(Self::find_memory_type(
                mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?);
        
        let buffer_memory = unsafe { device.allocate_memory(&alloc_info, None)? };
        
        unsafe {
            device.bind_buffer_memory(buffer, buffer_memory, 0)?;
        }
        
        Ok((buffer, buffer_memory))
    }
    
    #[allow(dead_code)]
    fn create_index_buffer(
        device: &Device,
        indices: &[u32],
    ) -> Result<(vk::Buffer, vk::DeviceMemory), Box<dyn std::error::Error>> {
        let buffer_size = (std::mem::size_of::<u32>() * indices.len()) as vk::DeviceSize;
        
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        
        let buffer = unsafe { device.create_buffer(&buffer_info, None)? };
        
        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(Self::find_memory_type(
                mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?);
        
        let buffer_memory = unsafe { device.allocate_memory(&alloc_info, None)? };
        
        unsafe {
            device.bind_buffer_memory(buffer, buffer_memory, 0)?;
        }
        
        Ok((buffer, buffer_memory))
    }
    
    #[allow(dead_code)]
    fn find_memory_type(
        type_filter: u32,
        _properties: vk::MemoryPropertyFlags,
    ) -> Result<u32, Box<dyn std::error::Error>> {
        // This is a simplified implementation
        // In a real application, you would query the physical device memory properties
        // For now, we'll just return the first memory type that matches
        for i in 0..32 {
            if (type_filter & (1 << i)) != 0 {
                // In a real implementation, you would check if the memory type has the required properties
                return Ok(i);
            }
        }
        Err("Failed to find suitable memory type".into())
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