use ash::vk;
use ash::Device;
use std::ffi::CStr;

pub struct VulkanPipeline {
    pub render_pass: vk::RenderPass,
    pub pipeline_layout: vk::PipelineLayout,
    pub graphics_pipeline: vk::Pipeline,
    pub device: Device, // Add device reference for cleanup
}

impl VulkanPipeline {
    pub fn new(device: &Device, swapchain_format: vk::Format) -> Result<Self, Box<dyn std::error::Error>> {
        let render_pass = Self::create_render_pass(device, swapchain_format)?;
        let (pipeline_layout, graphics_pipeline) = Self::create_graphics_pipeline(device, render_pass)?;
        
        Ok(Self {
            render_pass,
            pipeline_layout,
            graphics_pipeline,
            device: device.clone(), // Clone device for cleanup
        })
    }
    
    fn create_render_pass(device: &Device, format: vk::Format) -> Result<vk::RenderPass, Box<dyn std::error::Error>> {
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
        
        let render_pass = unsafe { device.create_render_pass(&render_pass_info, None)? };
        Ok(render_pass)
    }
    
    fn create_graphics_pipeline(
        device: &Device, 
        render_pass: vk::RenderPass
    ) -> Result<(vk::PipelineLayout, vk::Pipeline), Box<dyn std::error::Error>> {
        // Load shaders
        let vert_shader_code = include_bytes!("../../shaders/triangle.vert.spv");
        let frag_shader_code = include_bytes!("../../shaders/triangle.frag.spv");
        
        if vert_shader_code.is_empty() || frag_shader_code.is_empty() {
            return Err("Shader files are empty. Please compile GLSL shaders to SPIR-V using glslc.".into());
        }
        
        let vert_shader_module = Self::create_shader_module(device, vert_shader_code)?;
        let frag_shader_module = Self::create_shader_module(device, frag_shader_code)?;
        
        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") });
        
        let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") });
        
        let shader_stages = [vert_stage.build(), frag_stage.build()];
        
        // Vertex input (empty for now)
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder();
        
        // Input assembly
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
        
        // Viewport and scissor
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: 800,
                height: 600,
            },
        };
        
        let viewports = [viewport];
        let scissors = [scissor];
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);
        
        // Rasterizer
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
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
        
        // Pipeline layout
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder();
        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };
        
        // Graphics pipeline
        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);
        
        let graphics_pipeline = unsafe {
            let result = device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info.build()], None);
            match result {
                Ok(pipelines) => pipelines[0],
                Err((_, result)) => return Err(format!("Failed to create graphics pipeline: {:?}", result).into()),
            }
        };
        
        // Cleanup shader modules
        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }
        
        Ok((pipeline_layout, graphics_pipeline))
    }
    
    fn create_shader_module(device: &Device, code: &[u8]) -> Result<vk::ShaderModule, Box<dyn std::error::Error>> {
        let create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: code.len(),
            p_code: code.as_ptr() as *const u32,
        };
        
        let shader_module = unsafe { device.create_shader_module(&create_info, None)? };
        Ok(shader_module)
    }
}

impl Drop for VulkanPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
        }
    }
}