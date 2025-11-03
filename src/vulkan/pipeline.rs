use ash::vk;
use ash::Device;
use std::ffi::CStr;
use crate::error::{Result, VulkanError};
use crate::config;
use log::{debug, info};

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
        
        let render_pass = Self::create_render_pass(device, swapchain_format)?;
        debug!("Render pass created successfully");
        
        let (pipeline_layout, graphics_pipeline) = Self::create_graphics_pipeline(device, render_pass)?;
        debug!("Graphics pipeline created successfully");
        
        info!("Vulkan pipeline created successfully");
        
        Ok(Self {
            render_pass,
            pipeline_layout,
            graphics_pipeline,
            device: device.clone(), // Clone device for cleanup
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
        
        let color_attachment = vk::AttachmentDescription::builder()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();
        
        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();
        
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[color_attachment_ref])
            .build();
        
        let attachments = [color_attachment];
        let subpasses = [subpass];
        let render_pass_info = vk::RenderPassCreateInfo::builder()
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
        render_pass: vk::RenderPass
    ) -> Result<(vk::PipelineLayout, vk::Pipeline)> {
        debug!("Creating graphics pipeline");
        
        // Load SDF shaders
        let vert_shader_code = include_bytes!("../../shaders/sdf.vert.spv");
        let frag_shader_code = include_bytes!("../../shaders/sdf.frag.spv");
        
        if vert_shader_code.is_empty() || frag_shader_code.is_empty() {
            return Err(VulkanError::ShaderCompilation(
                "Shader files are empty. Please compile GLSL shaders to SPIR-V using glslc.".to_string()
            ).into());
        }
        
        debug!("Loading vertex shader ({} bytes)", vert_shader_code.len());
        debug!("Loading fragment shader ({} bytes)", frag_shader_code.len());
        
        let vert_shader_module = Self::create_shader_module(device, vert_shader_code)?;
        let frag_shader_module = Self::create_shader_module(device, frag_shader_code)?;
        
        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(unsafe { CStr::from_bytes_with_nul_unchecked(config::shader::ENTRY_POINT) });
        
        let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(unsafe { CStr::from_bytes_with_nul_unchecked(config::shader::ENTRY_POINT) });
        
        let shader_stages = [vert_stage.build(), frag_stage.build()];
        
        // Vertex input (empty for now)
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder();
        
        // Input assembly
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
        
        // Dynamic viewport and scissor (will be set at command buffer time)
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);
        
        // Rasterizer
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(config::rendering::LINE_WIDTH)
            .cull_mode(if config::rendering::ENABLE_FACE_CULLING { config::rendering::CULL_MODE } else { vk::CullModeFlags::NONE })
            .front_face(config::rendering::FRONT_FACE)
            .depth_bias_enable(false);
        
        // Multisampling
        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        
        // Color blending
        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A)
            .blend_enable(false)
            .build();
        
        let color_blend_attachments = [color_blend_attachment_state];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .attachments(&color_blend_attachments);
        
        // Push constant range for window data
        let push_constant_range = vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: 16, // vec2 + float + float = 4 + 4 + 4 + 4 = 16 bytes
        };
        let push_constant_ranges = [push_constant_range];
        
        // Pipeline layout with push constants
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(&push_constant_ranges);
        let pipeline_layout = unsafe {
            device.create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|e| VulkanError::PipelineCreation(format!("Failed to create pipeline layout: {:?}", e)))?
        };
        
        // Dynamic states
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states);
        
        // Graphics pipeline
        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
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
            let result = device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info.build()], None);
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
        };
        
        let shader_module = unsafe {
            device.create_shader_module(&create_info, None)
                .map_err(|e| VulkanError::ShaderCompilation(format!("Failed to create shader module: {:?}", e)))?
        };
        
        debug!("Shader module created successfully");
        Ok(shader_module)
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