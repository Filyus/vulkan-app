use legion::{Resources, Schedule, World};
use crate::ecs::systems::{create_triangle_mesh, render_system, transform_update_system};
use crate::vulkan::renderer::VulkanRenderer;

pub struct ECSWorld {
    pub world: World,
    pub resources: Resources,
    pub schedule: Schedule,
}

impl ECSWorld {
    pub fn new(vulkan_renderer: VulkanRenderer) -> Self {
        let mut world = World::default();
        let mut resources = Resources::default();
        
        // Insert the Vulkan renderer as a resource
        resources.insert(vulkan_renderer);
        
        // Insert a vector to track triangle entities
        resources.insert(Vec::<legion::Entity>::new());
        
        // Create the triangle mesh once during initialization
        create_triangle_mesh(&mut world, &mut resources);
        
        // Create the schedule with systems that run every frame
        let schedule = Schedule::builder()
            .add_thread_local_fn(transform_update_system)
            .add_thread_local_fn(render_system)
            .build();
        
        Self {
            world,
            resources,
            schedule,
        }
    }
    
    pub fn execute(&mut self) {
        self.schedule.execute(&mut self.world, &mut self.resources);
    }
    
    pub fn draw_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut vulkan_renderer = self.resources.get_mut::<VulkanRenderer>()
            .expect("VulkanRenderer resource not found in ECS world");
        vulkan_renderer.draw_frame()
    }
}