use ash::vk;
use ash::Device;
use log::{debug, info, warn, error};
use crate::error::AppError;
use std::mem;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ImguiVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub col: [u8; 4],
}

pub struct ImGuiVulkanBackend {
    device: Device,
    physical_device: Option<vk::PhysicalDevice>,
    instance: Option<ash::Instance>,
    graphics_queue_family_index: u32,
    font_texture: Option<vk::Image>,
    font_texture_view: Option<vk::ImageView>,
    font_texture_sampler: Option<vk::Sampler>,
    font_texture_memory: Option<vk::DeviceMemory>,
    descriptor_set_layout: Option<vk::DescriptorSetLayout>,
    descriptor_pool: Option<vk::DescriptorPool>,
    descriptor_set: Option<vk::DescriptorSet>,
    pipeline_layout: Option<vk::PipelineLayout>,
    pipeline: Option<vk::Pipeline>,
    vertex_buffer: Option<vk::Buffer>,
    vertex_buffer_memory: Option<vk::DeviceMemory>,
    index_buffer: Option<vk::Buffer>,
    index_buffer_memory: Option<vk::DeviceMemory>,
    vertex_count: usize,
    index_count: usize,
}

impl ImGuiVulkanBackend {
    pub fn new(
        device: &Device,
        physical_device: vk::PhysicalDevice,
        instance: &ash::Instance,
        render_pass: vk::RenderPass,
        graphics_queue_family_index: u32,
    ) -> Result<Self, AppError> {
        let mut backend = Self {
            device: device.clone(),
            physical_device: Some(physical_device),
            instance: Some(instance.clone()),
            graphics_queue_family_index,
            font_texture: None,
            font_texture_view: None,
            font_texture_sampler: None,
            font_texture_memory: None,
            descriptor_set_layout: None,
            descriptor_pool: None,
            descriptor_set: None,
            pipeline_layout: None,
            pipeline: None,
            vertex_buffer: None,
            vertex_buffer_memory: None,
            index_buffer: None,
            index_buffer_memory: None,
            vertex_count: 0,
            index_count: 0,
        };

        // Create descriptor set layout
        backend.create_descriptor_set_layout()?;
        
        // Create pipeline
        backend.create_pipeline(render_pass)?;
        
        // Create descriptor pool
        backend.create_descriptor_pool()?;
        
        // Allocate descriptor set
        backend.allocate_descriptor_set()?;

        info!("ImGui Vulkan backend created successfully");
        Ok(backend)
    }

    fn create_descriptor_set_layout(&mut self) -> Result<(), AppError> {
        let binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let bindings = [binding];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings);

        self.descriptor_set_layout = unsafe {
            Some(self.device.create_descriptor_set_layout(&layout_info, None)?)
        };

        Ok(())
    }

    fn create_pipeline(&mut self, render_pass: vk::RenderPass) -> Result<(), AppError> {
        // Create pipeline layout with push constants
        let descriptor_set_layout = self.descriptor_set_layout.unwrap();
        let descriptor_set_layout_array = [descriptor_set_layout];
        
        // Define push constant range for vertex shader (projection matrix)
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(64); // 4x4 matrix = 16 floats = 64 bytes
        
        let push_constant_ranges = [push_constant_range];
        
        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&descriptor_set_layout_array)
            .push_constant_ranges(&push_constant_ranges);

        self.pipeline_layout = unsafe {
            Some(self.device.create_pipeline_layout(&layout_info, None)?)
        };

        // Create shader modules from compiled SPIR-V
        let vert_shader_code = include_bytes!("../../shaders/imgui.vert.spv");
        let frag_shader_code = include_bytes!("../../shaders/imgui.frag.spv");

        let vert_shader_module = Self::create_shader_module_from_spv(&self.device, vert_shader_code)?;
        let frag_shader_module = Self::create_shader_module_from_spv(&self.device, frag_shader_code)?;

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_shader_module)
                .name(c"main"),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_shader_module)
                .name(c"main"),
        ];

        // Vertex input
        let binding = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(mem::size_of::<ImguiVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX);

        let attribute_descriptions = [
            vk::VertexInputAttributeDescription::default()
                .location(0)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(0), // Position
            vk::VertexInputAttributeDescription::default()
                .location(1)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(mem::size_of::<[f32; 2]>() as u32), // UV
            vk::VertexInputAttributeDescription::default()
                .location(2)
                .binding(0)
                .format(vk::Format::R8G8B8A8_UNORM)
                .offset((mem::size_of::<[f32; 2]>() * 2) as u32), // Color
        ];

        let binding_array = [binding];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding_array)
            .vertex_attribute_descriptions(&attribute_descriptions);

        // Input assembly
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        // Viewport - will be set dynamically during rendering
        let viewport = vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(1.0)  // Placeholder - will be set during render
            .height(1.0) // Placeholder - will be set during render
            .min_depth(0.0)
            .max_depth(1.0);

        let _scissor = vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(vk::Extent2D { width: 1, height: 1 }); // Placeholder - will be set during render

        let _viewport_array = [viewport];
        // Don't set viewport and scissor in pipeline since we'll set them dynamically
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&[])  // Empty because we'll set dynamically
            .scissors(&[]);  // Empty because we'll set dynamically

        // Dynamic state for viewport and scissor
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&dynamic_states);

        // Rasterization
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        // Multisampling
        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        // Color blending
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);

        let color_blend_attachment_array = [color_blend_attachment];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(&color_blend_attachment_array);

        // Pipeline
        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state_info)
            .layout(self.pipeline_layout.unwrap())
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = unsafe {
            self.device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|(_, e)| e)?[0]
        };

        self.pipeline = Some(pipeline);

        // Cleanup shader modules
        unsafe {
            self.device.destroy_shader_module(vert_shader_module, None);
            self.device.destroy_shader_module(frag_shader_module, None);
        }

        Ok(())
    }

    fn create_shader_module_from_spv(device: &Device, spv_code: &[u8]) -> Result<vk::ShaderModule, AppError> {
        // SPIR-V is already aligned to 4 bytes, just need to cast to u32
        let aligned_code: Vec<u32> = spv_code
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        let shader_module_create_info = vk::ShaderModuleCreateInfo::default()
            .code(&aligned_code);

        unsafe {
            Ok(device.create_shader_module(&shader_module_create_info, None)?)
        }
    }

    fn create_descriptor_pool(&mut self) -> Result<(), AppError> {
        let pool_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1);

        let pool_sizes = [pool_size];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(1);

        self.descriptor_pool = unsafe {
            Some(self.device.create_descriptor_pool(&pool_info, None)?)
        };

        Ok(())
    }

    fn allocate_descriptor_set(&mut self) -> Result<(), AppError> {
        let layouts = [self.descriptor_set_layout.unwrap()];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.descriptor_pool.unwrap())
            .set_layouts(&layouts);

        let descriptor_sets = unsafe {
            self.device.allocate_descriptor_sets(&alloc_info)?
        };

        self.descriptor_set = Some(descriptor_sets[0]);
        Ok(())
    }

    pub fn create_font_texture(&mut self, width: u32, height: u32) -> Result<(), AppError> {
        debug!("Creating font texture {}x{}", width, height);

        // Ensure minimum size for font texture
        let texture_width = std::cmp::max(width, 1);
        let texture_height = std::cmp::max(height, 1);
        
        // Create font texture with RGBA format for proper font rendering
        // Use OPTIMAL tiling for better GPU performance and proper sampling
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D { width: texture_width, height: texture_height, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8G8B8A8_UNORM) // Use RGBA format for proper font rendering
            .tiling(vk::ImageTiling::OPTIMAL) // Use OPTIMAL tiling for GPU sampling
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        self.font_texture = unsafe {
            Some(self.device.create_image(&image_info, None)?)
        };

        // Allocate memory for the texture
        let mem_requirements = unsafe { self.device.get_image_memory_requirements(self.font_texture.unwrap()) };
        
        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(self.find_memory_type(
                mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?);

        self.font_texture_memory = unsafe {
            Some(self.device.allocate_memory(&alloc_info, None)?)
        };

        unsafe {
            self.device.bind_image_memory(self.font_texture.unwrap(), self.font_texture_memory.unwrap(), 0)?;
        }

        debug!("Font texture image created with optimal tiling, size: {}x{}", texture_width, texture_height);

        // Create image view
        let view_info = vk::ImageViewCreateInfo::default()
            .image(self.font_texture.unwrap())
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        self.font_texture_view = unsafe {
            Some(self.device.create_image_view(&view_info, None)?)
        };

        // Create sampler
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .anisotropy_enable(false)
            .max_anisotropy(1.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(vk::LOD_CLAMP_NONE);

        self.font_texture_sampler = unsafe {
            Some(self.device.create_sampler(&sampler_info, None)?)
        };

        debug!("Font texture image view and sampler created");

        // Update descriptor set
        let descriptor_image_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.font_texture_view.unwrap())
            .sampler(self.font_texture_sampler.unwrap());

        let descriptor_image_info_array = [descriptor_image_info];
        let write_descriptor_set = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set.unwrap())
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&descriptor_image_info_array);

        unsafe {
            self.device.update_descriptor_sets(&[write_descriptor_set], &[]);
        }

        debug!("Font texture descriptor set updated successfully");
        Ok(())
    }

    pub fn upload_font_data(&mut self, width: u32, height: u32, pixels: &[u8]) -> Result<(), AppError> {
        info!("Uploading font data {}x{} ({} bytes)", width, height, pixels.len());

        if pixels.is_empty() {
            warn!("Font pixel data is empty!");
            return Ok(());
        }

        info!("Font data received: {}x{} pixels, {} bytes total", width, height, pixels.len());
        
        // Check first few pixels to verify data
        if pixels.len() >= 4 {
            info!("First pixel values: [{}, {}, {}, {}]", pixels[0], pixels[1], pixels[2], pixels[3]);
        }
        
        // Verify font texture exists
        if self.font_texture.is_none() {
            error!("Font texture not created yet!");
            return Err(AppError::HUD("Font texture not created".to_string()));
        }
        
        info!("Font texture exists, proceeding with upload");

        // Create a staging buffer for uploading the font data
        let buffer_size = (width * height * 4) as u64; // 4 bytes per pixel for RGBA
        
        let staging_buffer_info = vk::BufferCreateInfo::default()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let staging_buffer = unsafe {
            self.device.create_buffer(&staging_buffer_info, None)?
        };

        let staging_mem_requirements = unsafe { self.device.get_buffer_memory_requirements(staging_buffer) };
        
        let staging_alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(staging_mem_requirements.size)
            .memory_type_index(self.find_memory_type(
                staging_mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?);

        let staging_buffer_memory = unsafe {
            self.device.allocate_memory(&staging_alloc_info, None)?
        };

        unsafe {
            self.device.bind_buffer_memory(staging_buffer, staging_buffer_memory, 0)?;
        }

        // Map the staging buffer and copy font data
        let mapped_memory = unsafe {
            self.device.map_memory(
                staging_buffer_memory,
                0,
                buffer_size,
                vk::MemoryMapFlags::empty(),
            )?
        };

        unsafe {
            let copy_size = std::cmp::min(pixels.len(), buffer_size as usize);
            std::ptr::copy_nonoverlapping(pixels.as_ptr(), mapped_memory as *mut u8, copy_size);
            self.device.unmap_memory(staging_buffer_memory);
        }

        debug!("Font data copied to staging buffer");

        // Create a temporary command buffer for the texture upload
        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(self.graphics_queue_family_index);

        let command_pool = unsafe {
            self.device.create_command_pool(&command_pool_info, None)?
        };

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffers = unsafe {
            self.device.allocate_command_buffers(&alloc_info)?
        };
        let command_buffer = command_buffers[0];

        // Begin command buffer
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device.begin_command_buffer(command_buffer, &begin_info)?;
        }

        // Transition image layout to TRANSFER_DST_OPTIMAL
        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(self.font_texture.unwrap())
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        unsafe {
            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        }

        // Copy buffer to image
        let region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D { width, height, depth: 1 });

        unsafe {
            self.device.cmd_copy_buffer_to_image(
                command_buffer,
                staging_buffer,
                self.font_texture.unwrap(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        // Transition image layout to SHADER_READ_ONLY_OPTIMAL
        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(self.font_texture.unwrap())
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        unsafe {
            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        }

        // End and submit command buffer
        unsafe {
            self.device.end_command_buffer(command_buffer)?;
        }

        let command_buffers_array = [command_buffer];
        let submit_info = vk::SubmitInfo::default()
            .command_buffers(&command_buffers_array);

        unsafe {
            self.device.queue_submit(self.device.get_device_queue(self.graphics_queue_family_index, 0), &[submit_info], vk::Fence::null())?;
            self.device.queue_wait_idle(self.device.get_device_queue(self.graphics_queue_family_index, 0))?;
        }

        // Cleanup
        unsafe {
            self.device.free_command_buffers(command_pool, &[command_buffer]);
            self.device.destroy_command_pool(command_pool, None);
            self.device.destroy_buffer(staging_buffer, None);
            self.device.free_memory(staging_buffer_memory, None);
        }

        info!("Font texture upload completed successfully with proper layout transitions");
        
        // Verify the texture was uploaded correctly by checking descriptor set
        if self.descriptor_set.is_some() {
            info!("Font texture descriptor set is bound and ready");
        } else {
            error!("Font texture descriptor set is not bound!");
        }
        
        Ok(())
    }

    fn find_memory_type(&self, type_filter: u32, properties: vk::MemoryPropertyFlags) -> Result<u32, AppError> {
        if let (Some(physical_device), Some(instance)) = (self.physical_device, &self.instance) {
            let mem_properties = unsafe {
                instance.get_physical_device_memory_properties(physical_device)
            };
            
            for i in 0..mem_properties.memory_type_count {
                if (type_filter & (1 << i)) != 0 &&
                   mem_properties.memory_types[i as usize].property_flags.contains(properties) {
                    debug!("Found suitable memory type {} for properties {:?}", i, properties);
                    return Ok(i);
                }
            }
        }
        
        error!("Failed to find suitable memory type for filter {:032b} and properties {:?}", type_filter, properties);
        Err(AppError::HUD("Failed to find suitable memory type".to_string()))
    }

    pub fn render(&mut self, draw_data: &imgui::DrawData, command_buffer: vk::CommandBuffer) -> Result<(), AppError> {
        info!("Rendering ImGui with {} draw lists", draw_data.draw_lists().count());
        
        // Verify font texture is ready
        if self.font_texture.is_none() || self.font_texture_view.is_none() || self.descriptor_set.is_none() {
            error!("Font texture not properly initialized for rendering!");
            return Err(AppError::HUD("Font texture not properly initialized".to_string()));
        }
        
        info!("Font texture is properly initialized, proceeding with rendering");

        // Create vertex and index buffers
        self.create_buffers(draw_data)?;

        // Setup projection matrix push constants
        // ImGui uses clip space coordinates: (0,0) = top-left, (width,height) = bottom-right
        // Vulkan uses: (-1,-1) = top-left, (1,1) = bottom-right
        let [width, height] = draw_data.display_size;
        let ortho = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, 2.0 / height, 0.0, 0.0],  // Positive Y for Vulkan's coordinate system
            [0.0, 0.0, -1.0, 0.0],
            [-1.0, -1.0, 0.0, 1.0],  // Map (0,0) to (-1,-1) top-left corner
        ];

        unsafe {
            // Push projection matrix
            self.device.cmd_push_constants(
                command_buffer,
                self.pipeline_layout.unwrap(),
                vk::ShaderStageFlags::VERTEX,
                0,
                bytemuck::bytes_of(&ortho),
            );

            // Bind pipeline
            self.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.unwrap());

            // Bind descriptor set
            info!("Binding font texture descriptor set");
            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout.unwrap(),
                0,
                &[self.descriptor_set.unwrap()],
                &[],
            );
            info!("Descriptor set bound successfully");

            // Bind vertex and index buffers
            if let (Some(vertex_buffer), Some(index_buffer)) = (self.vertex_buffer, self.index_buffer) {
                self.device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
                self.device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT16);
            }

            // Set viewport and scissor
            let viewport = vk::Viewport::default()
                .x(0.0)
                .y(0.0)
                .width(width)
                .height(height)
                .min_depth(0.0)
                .max_depth(1.0);

            let scissor = vk::Rect2D::default()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(vk::Extent2D {
                    width: width as u32,
                    height: height as u32,
                });

            self.device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            self.device.cmd_set_scissor(command_buffer, 0, &[scissor]);
        }

        // Draw each list
        let mut index_offset = 0;
        let mut vertex_offset = 0;

        for (i, draw_list) in draw_data.draw_lists().enumerate() {
            debug!("Rendering draw list {} with {} vertices and {} indices",
                   i, draw_list.vtx_buffer().len(), draw_list.idx_buffer().len());

            unsafe {
                self.device.cmd_draw_indexed(
                    command_buffer,
                    draw_list.idx_buffer().len() as u32,
                    1,
                    index_offset as u32,
                    vertex_offset as i32,
                    0,
                );
            }

            index_offset += draw_list.idx_buffer().len();
            vertex_offset += draw_list.vtx_buffer().len();
        }

        Ok(())
    }

    /// Clean up dynamic buffers after rendering
    /// This should be called after each frame to ensure buffers are properly destroyed
    pub fn cleanup_dynamic_buffers(&mut self) {
        debug!("Cleaning up dynamic ImGui buffers");
        
        unsafe {
            if let Some(vertex_buffer) = self.vertex_buffer {
                self.device.destroy_buffer(vertex_buffer, None);
            }
            if let Some(vertex_memory) = self.vertex_buffer_memory {
                self.device.free_memory(vertex_memory, None);
            }
            if let Some(index_buffer) = self.index_buffer {
                self.device.destroy_buffer(index_buffer, None);
            }
            if let Some(index_memory) = self.index_buffer_memory {
                self.device.free_memory(index_memory, None);
            }
        }
        
        self.vertex_buffer = None;
        self.vertex_buffer_memory = None;
        self.index_buffer = None;
        self.index_buffer_memory = None;
        self.vertex_count = 0;
        self.index_count = 0;
        
        debug!("Dynamic ImGui buffers cleaned up");
    }

    fn create_buffers(&mut self, draw_data: &imgui::DrawData) -> Result<(), AppError> {
        // Calculate total vertex and index counts
        let mut total_vertices = 0;
        let mut total_indices = 0;

        for draw_list in draw_data.draw_lists() {
            total_vertices += draw_list.vtx_buffer().len();
            total_indices += draw_list.idx_buffer().len();
        }

        if total_vertices == 0 || total_indices == 0 {
            return Ok(());
        }

        // Clean up existing buffers before creating new ones
        self.cleanup_dynamic_buffers();

        // Create vertex buffer
        let vertex_buffer_size = (total_vertices * mem::size_of::<ImguiVertex>()) as u64;
        let vertex_buffer_info = vk::BufferCreateInfo::default()
            .size(vertex_buffer_size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        self.vertex_buffer = unsafe {
            Some(self.device.create_buffer(&vertex_buffer_info, None)?)
        };

        // Create index buffer
        let index_buffer_size = (total_indices * mem::size_of::<u16>()) as u64;
        let index_buffer_info = vk::BufferCreateInfo::default()
            .size(index_buffer_size)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        self.index_buffer = unsafe {
            Some(self.device.create_buffer(&index_buffer_info, None)?)
        };

        // Allocate memory for vertex buffer
        let vertex_mem_requirements = unsafe { self.device.get_buffer_memory_requirements(self.vertex_buffer.unwrap()) };
        debug!("Vertex buffer memory requirements: size={}, type_bits={:032b}", vertex_mem_requirements.size, vertex_mem_requirements.memory_type_bits);
        
        let vertex_alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(vertex_mem_requirements.size)
            .memory_type_index(self.find_memory_type(
                vertex_mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?);

        self.vertex_buffer_memory = unsafe {
            Some(self.device.allocate_memory(&vertex_alloc_info, None)?)
        };

        unsafe {
            self.device.bind_buffer_memory(self.vertex_buffer.unwrap(), self.vertex_buffer_memory.unwrap(), 0)?;
        }

        // Allocate memory for index buffer
        let index_mem_requirements = unsafe { self.device.get_buffer_memory_requirements(self.index_buffer.unwrap()) };
        debug!("Index buffer memory requirements: size={}, type_bits={:032b}", index_mem_requirements.size, index_mem_requirements.memory_type_bits);
        
        let index_alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(index_mem_requirements.size)
            .memory_type_index(self.find_memory_type(
                index_mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?);

        self.index_buffer_memory = unsafe {
            Some(self.device.allocate_memory(&index_alloc_info, None)?)
        };

        unsafe {
            self.device.bind_buffer_memory(self.index_buffer.unwrap(), self.index_buffer_memory.unwrap(), 0)?;
        }

        // Upload vertex data - map the entire buffer once
        debug!("Mapping vertex buffer memory: size={}, buffer={:?}", vertex_buffer_size, self.vertex_buffer.unwrap());
        let vertex_mapped_memory = unsafe {
            self.device.map_memory(
                self.vertex_buffer_memory.unwrap(),
                0,
                vertex_buffer_size,
                vk::MemoryMapFlags::empty(),
            )?
        };
        debug!("Vertex buffer memory mapped successfully");
        
        let mut vertex_offset = 0;
        for (list_idx, draw_list) in draw_data.draw_lists().enumerate() {
            let vertices = draw_list.vtx_buffer();
            let vertex_size = vertices.len() * mem::size_of::<ImguiVertex>();
            
            if vertex_size > 0 {
                debug!("Processing draw list {} with {} vertices", list_idx, vertices.len());
                
                // Log first few vertices for debugging
                for (i, vertex) in vertices.iter().take(3).enumerate() {
                    debug!("Vertex {}: pos=({:.2},{:.2}), uv=({:.3},{:.3}), col=({},{},{},{})",
                           i, vertex.pos[0], vertex.pos[1], vertex.uv[0], vertex.uv[1],
                           vertex.col[0], vertex.col[1], vertex.col[2], vertex.col[3]);
                }
                
                unsafe {
                    let dst = vertex_mapped_memory.add(vertex_offset) as *mut ImguiVertex;
                    // Convert DrawVert to ImguiVertex
                    for (i, vertex) in vertices.iter().enumerate() {
                        let imgui_vertex = ImguiVertex {
                            pos: [vertex.pos[0], vertex.pos[1]],
                            uv: [vertex.uv[0], vertex.uv[1]],
                            col: [
                                vertex.col[0],
                                vertex.col[1],
                                vertex.col[2],
                                vertex.col[3],
                            ],
                        };
                        dst.add(i).write(imgui_vertex);
                    }
                }
            }
            
            vertex_offset += vertex_size;
        }
        
        // Unmap vertex memory
        unsafe {
            self.device.unmap_memory(self.vertex_buffer_memory.unwrap());
        }

        // Upload index data - map the entire buffer once
        debug!("Mapping index buffer memory: size={}, buffer={:?}", index_buffer_size, self.index_buffer.unwrap());
        let index_mapped_memory = unsafe {
            self.device.map_memory(
                self.index_buffer_memory.unwrap(),
                0,
                index_buffer_size,
                vk::MemoryMapFlags::empty(),
            )?
        };
        debug!("Index buffer memory mapped successfully");
        
        let mut index_offset = 0;
        for draw_list in draw_data.draw_lists() {
            let indices = draw_list.idx_buffer();
            let index_size = indices.len() * mem::size_of::<u16>();
            
            if index_size > 0 {
                unsafe {
                    let dst = index_mapped_memory.add(index_offset) as *mut u16;
                    dst.copy_from_nonoverlapping(indices.as_ptr(), indices.len());
                }
            }
            
            index_offset += index_size;
        }
        
        // Unmap index memory
        unsafe {
            self.device.unmap_memory(self.index_buffer_memory.unwrap());
        }
        
        debug!("Uploaded {} vertices and {} indices to GPU buffers", total_vertices, total_indices);
        self.vertex_count = total_vertices;
        self.index_count = total_indices;

        Ok(())
    }

    pub fn cleanup(&mut self) {
        debug!("Cleaning up ImGui Vulkan backend");
        
        // First clean up dynamic buffers
        self.cleanup_dynamic_buffers();
        
        unsafe {
            if let Some(pipeline) = self.pipeline {
                self.device.destroy_pipeline(pipeline, None);
            }
            if let Some(pipeline_layout) = self.pipeline_layout {
                self.device.destroy_pipeline_layout(pipeline_layout, None);
            }
            if let Some(sampler) = self.font_texture_sampler {
                self.device.destroy_sampler(sampler, None);
            }
            if let Some(view) = self.font_texture_view {
                self.device.destroy_image_view(view, None);
            }
            if let Some(image) = self.font_texture {
                self.device.destroy_image(image, None);
            }
            if let Some(memory) = self.font_texture_memory {
                self.device.free_memory(memory, None);
            }
            if let Some(pool) = self.descriptor_pool {
                self.device.destroy_descriptor_pool(pool, None);
            }
            if let Some(layout) = self.descriptor_set_layout {
                self.device.destroy_descriptor_set_layout(layout, None);
            }
        }
        
        self.font_texture = None;
        self.font_texture_view = None;
        self.font_texture_sampler = None;
        self.font_texture_memory = None;
        self.descriptor_set_layout = None;
        self.descriptor_pool = None;
        self.descriptor_set = None;
        self.pipeline_layout = None;
        self.pipeline = None;
        
        debug!("ImGui Vulkan backend cleanup completed");
    }
}

impl Drop for ImGuiVulkanBackend {
    fn drop(&mut self) {
        self.cleanup();
    }
}