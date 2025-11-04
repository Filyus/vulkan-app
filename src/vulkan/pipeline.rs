use ash::vk;
use ash::Device;
use std::ffi::CStr;
use crate::error::{Result, VulkanError};
use crate::config;
use crate::vulkan::shader_compiler::ShaderCompiler;
use log::{debug, info, warn};

/// Vulkan pipeline wrapper with proper resource management
///
/// This struct manages the Vulkan render pass, pipeline layout, and graphics pipeline,
/// ensuring proper cleanup and providing debugging capabilities.
pub struct VulkanPipeline {
    /// The render pass
    pub render_pass: vk::RenderPass,
    
    /// The pipeline layout
    pub pipeline_layout: vk::PipelineLayout,
    
    /// The graphics pipeline
    pub graphics_pipeline: vk::Pipeline,
    
    /// The device reference for cleanup
    pub device: Device,
    
    /// Shader compiler for runtime compilation
    #[allow(dead_code)]
    shader_compiler: ShaderCompiler,
}

impl VulkanPipeline {
    /// Create a new Vulkan pipeline
    ///
    /// # Arguments
    /// * `device` - The Vulkan device
    /// * `swapchain_format` - The swapchain image format
    ///
    /// # Returns
    /// A new VulkanPipeline instance
    ///
    /// # Errors
    /// Returns an error if pipeline creation fails
    pub fn new(device: &Device, swapchain_format: vk::Format) -> Result<Self> {
        info!("Creating Vulkan pipeline");
        
        // Initialize shader compiler
        let mut shader_compiler = ShaderCompiler::new()?;
        
        // Configure shader compiler based on settings
        shader_compiler.configure(
            config::shader::ENABLE_SHADER_CACHE,
            config::shader::ENABLE_SHADER_DEBUG,
            config::shader::OPTIMIZATION_LEVEL
        );
        
        // Preload shaders if enabled
        if config::shader::PRELOAD_SHADERS {
            info!("Preloading shaders...");
            let shaders_to_preload = [
                config::shader::SDF_VERTEX_SHADER,
                config::shader::SDF_FRAGMENT_SHADER,
                config::shader::IMGUI_VERTEX_SHADER,
                config::shader::IMGUI_FRAGMENT_SHADER,
            ];
            
            if let Err(e) = shader_compiler.preload_shaders(&shaders_to_preload) {
                warn!("Failed to preload some shaders: {}. Continuing with on-demand compilation.", e);
            } else {
                info!("Shader preloading completed successfully");
            }
        }
        
        let render_pass = Self::create_render_pass(device, swapchain_format)?;
        debug!("Render pass created successfully");
        
        let (pipeline_layout, graphics_pipeline) = Self::create_graphics_pipeline(device, render_pass, &mut shader_compiler)?;
        debug!("Graphics pipeline created successfully");
        
        info!("Vulkan pipeline created successfully with runtime shader compilation");
        
        Ok(Self {
            render_pass,
            pipeline_layout,
            graphics_pipeline,
            device: device.clone(), // Clone device for cleanup
            shader_compiler,
        })
    }
    
    /// Create a render pass
    ///
    /// # Arguments
    /// * `device` - The Vulkan device
    /// * `format` - The swapchain image format
    ///
    /// # Returns
    /// The created render pass
    ///
    /// # Errors
    /// Returns an error if render pass creation fails
    fn create_render_pass(device: &Device, format: vk::Format) -> Result<vk::RenderPass> {
        debug!("Creating render pass with format: {:?}", format);
        
        let color_attachment = vk::AttachmentDescription::default()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
        
        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        
        let color_attachment_refs = [color_attachment_ref];
        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs);
        
        let attachments = [color_attachment];
        let subpasses = [subpass];
        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses);
        
        let render_pass = unsafe {
            device.create_render_pass(&render_pass_info, None)
                .map_err(|e| VulkanError::PipelineCreation(format!("Failed to create render pass: {:?}", e)))?
        };
        
        debug!("Render pass created successfully");
        Ok(render_pass)
    }
    
    /// Create a graphics pipeline
    ///
    /// # Arguments
    /// * `device` - The Vulkan device
    /// * `render_pass` - The render pass
    ///
    /// # Returns
    /// A tuple of (pipeline_layout, graphics_pipeline)
    ///
    /// # Errors
    /// Returns an error if pipeline creation fails
    fn create_graphics_pipeline(
        device: &Device,
        render_pass: vk::RenderPass,
        shader_compiler: &mut ShaderCompiler
    ) -> Result<(vk::PipelineLayout, vk::Pipeline)> {
        debug!("Creating graphics pipeline with runtime shader compilation");
        
        // Compile shaders at runtime
        let vert_shader_code = shader_compiler.compile_file(
            config::shader::SDF_VERTEX_SHADER,
            "main"
        )?;
        
        let frag_shader_code = shader_compiler.compile_file(
            config::shader::SDF_FRAGMENT_SHADER,
            "main"
        )?;
        
        debug!("Compiled vertex shader ({} words)", vert_shader_code.len());
        debug!("Compiled fragment shader ({} words)", frag_shader_code.len());
        
        // Convert Vec<u32> to &[u8] for shader module creation
        let vert_shader_bytes = bytemuck::cast_slice(&vert_shader_code);
        let frag_shader_bytes = bytemuck::cast_slice(&frag_shader_code);
        
        let vert_shader_module = Self::create_shader_module(device, vert_shader_bytes)?;
        let frag_shader_module = Self::create_shader_module(device, frag_shader_bytes)?;
        
        let vert_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(unsafe { CStr::from_bytes_with_nul_unchecked(config::shader::ENTRY_POINT) });
        
        let frag_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(unsafe { CStr::from_bytes_with_nul_unchecked(config::shader::ENTRY_POINT) });
        
        let shader_stages = [vert_stage, frag_stage];
        
        // Vertex input (empty for now)
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();
        
        // Input assembly
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
        
        // Dynamic viewport and scissor (will be set at command buffer time)
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        
        // Rasterizer
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(config::rendering::LINE_WIDTH)
            .cull_mode(if config::rendering::ENABLE_FACE_CULLING { config::rendering::CULL_MODE } else { vk::CullModeFlags::NONE })
            .front_face(config::rendering::FRONT_FACE)
            .depth_bias_enable(false);
        
        // Multisampling
        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        
        // Color blending
        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A)
            .blend_enable(false);
        
        let color_blend_attachments = [color_blend_attachment_state];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(&color_blend_attachments);
        
        // Push constant range for window data (both vertex and fragment shaders)
        // Updated to match the actual push constant block size in the fragment shader (52 bytes)
        let push_constant_range = vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: 52, // Updated to match fragment shader push constant block size
        };
        let push_constant_ranges = [push_constant_range];
        
        // Pipeline layout with push constants
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .push_constant_ranges(&push_constant_ranges);
        let pipeline_layout = unsafe {
            device.create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|e| VulkanError::PipelineCreation(format!("Failed to create pipeline layout: {:?}", e)))?
        };
        
        // Dynamic states
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&dynamic_states);
        
        // Graphics pipeline
        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);
        
        let graphics_pipeline = unsafe {
            let result = device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None);
            match result {
                Ok(pipelines) => pipelines[0],
                Err((_, result)) => return Err(VulkanError::PipelineCreation(
                    format!("Failed to create graphics pipeline: {:?}", result)
                ).into()),
            }
        };
        
        // Cleanup shader modules
        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }
        
        debug!("Graphics pipeline created successfully");
        Ok((pipeline_layout, graphics_pipeline))
    }
    
    /// Create a shader module from SPIR-V code
    ///
    /// # Arguments
    /// * `device` - The Vulkan device
    /// * `code` - The SPIR-V shader code
    ///
    /// # Returns
    /// The created shader module
    ///
    /// # Errors
    /// Returns an error if shader module creation fails
    fn create_shader_module(device: &Device, code: &[u8]) -> Result<vk::ShaderModule> {
        let create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: code.len(),
            p_code: code.as_ptr() as *const u32,
            _marker: std::marker::PhantomData,
        };
        
        let shader_module = unsafe {
            device.create_shader_module(&create_info, None)
                .map_err(|e| VulkanError::ShaderCompilation(format!("Failed to create shader module: {:?}", e)))?
        };
        
        debug!("Shader module created successfully");
        Ok(shader_module)
    }
    
    /// Recompile shaders and recreate the pipeline
    ///
    /// This method allows for hot-reloading of shaders during development
    ///
    /// # Returns
    /// Ok(()) if recompilation succeeded
    /// Err if recompilation failed
    ///
    /// # Errors
    /// Returns an error if shader compilation or pipeline recreation fails
    #[allow(dead_code)]
    pub fn recompile_shaders(&mut self) -> Result<()> {
        info!("Recompiling shaders and recreating pipeline");
        
        // Clear shader cache to force recompilation
        self.shader_compiler.clear_cache();
        
        // Recreate the graphics pipeline with fresh shaders
        let (pipeline_layout, graphics_pipeline) = Self::create_graphics_pipeline(
            &self.device,
            self.render_pass,
            &mut self.shader_compiler
        )?;
        
        // Clean up old pipeline and layout
        unsafe {
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
        
        // Update with new pipeline
        self.pipeline_layout = pipeline_layout;
        self.graphics_pipeline = graphics_pipeline;
        
        info!("Shader recompilation completed successfully");
        Ok(())
    }
    
    /// Get shader compiler statistics
    ///
    /// # Returns
    /// Tuple of (cached_shaders, cache_size_bytes)
    #[allow(dead_code)]
    pub fn get_shader_cache_stats(&self) -> (usize, usize) {
        self.shader_compiler.get_cache_stats()
    }
}

impl Drop for VulkanPipeline {
    fn drop(&mut self) {
        debug!("Destroying Vulkan pipeline");
        unsafe {
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
        }
        debug!("Vulkan pipeline destroyed");
    }
}